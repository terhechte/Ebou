use std::collections::HashMap;

use crate::environment::model::Model;
use crate::view_model::StatusId;
use crate::view_model::StatusViewModel;
use id_tree::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    original_status: String,
    tree: Tree<StatusViewModel>,
}

impl PartialEq for Conversation {
    fn eq(&self, other: &Self) -> bool {
        self.original_status == other.original_status && self.tree == other.tree
    }
}

impl Eq for Conversation {}

#[derive(Debug, Clone, Copy)]
pub struct ConversationItem<'a> {
    node: &'a Node<StatusViewModel>,
    id: &'a NodeId,
}

impl<'a> ConversationItem<'a> {
    pub fn cloned_status(&self) -> StatusViewModel {
        self.node.data().clone()
    }
}

impl<'a> std::ops::Deref for ConversationItem<'a> {
    type Target = StatusViewModel;

    fn deref(&self) -> &Self::Target {
        self.node.data()
    }
}

impl Conversation {
    pub fn status(&self) -> StatusId {
        StatusId(self.original_status.clone())
    }

    pub fn root(&self) -> Option<ConversationItem<'_>> {
        let id = self.tree.root_node_id()?;
        let node = self.tree.get(id).ok()?;
        Some(ConversationItem { node, id })
    }

    pub fn children<'a>(&'a self, of: &ConversationItem) -> Option<Vec<ConversationItem<'a>>> {
        let children_ids = self.tree.children_ids(of.id).ok()?;
        Some(
            children_ids
                .filter_map(|e| self.tree.get(e).ok().map(|i| (e, i)))
                .map(|(id, node)| ConversationItem { node, id })
                .collect(),
        )
    }

    // insert as a child if the parent `id` exists and if `child.id`
    // doesn't exist yet as a child
    pub fn insert_child_if(&mut self, id: &StatusId, child: StatusViewModel) -> Option<bool> {
        use id_tree::InsertBehavior::*;
        let root_id = self.tree.root_node_id()?;
        let mut found_id = None;
        for node_id in self.tree.traverse_pre_order_ids(root_id).ok()? {
            let Ok(item) = self.tree.get(&node_id) else { continue };
            if &item.data().id == id {
                // check if this node already has the reply
                for child_id in item.children() {
                    let Ok(child_item) = self.tree.get(child_id) else {
                        continue
                    };
                    if child_item.data().id == child.id {
                        // we stop
                        return Some(false);
                    }
                }
                // otherwise, we found it and can insert
                found_id = Some(node_id);
                break;
            }
        }

        let found_id = found_id?;

        self.tree
            .insert(Node::new(child), UnderNode(&found_id))
            .ok()?;

        Some(true)
    }

    pub fn mutate_post<'a, C: FnMut(&'a mut StatusViewModel)>(
        &'a mut self,
        id: &StatusId,
        action: &'a mut C,
    ) -> bool {
        let Some(root_id) = self.tree.root_node_id() else {
            return false
        };
        let mut found = None;
        let Some(iter) = self.tree.traverse_pre_order_ids(root_id).ok() else {
            return false
        };
        for node_id in iter {
            if let Ok(item) = self.tree.get(&node_id) {
                if &item.data().id == id {
                    found = Some(node_id);
                    break;
                }
            }
        }

        if let Some(node_id) = found {
            if let Ok(item) = self.tree.get_mut(&node_id) {
                action(item.data_mut());
                return true;
            }
        }
        false
    }
}

pub async fn build_conversation(model: &Model, status_id: String) -> Result<Conversation, String> {
    let mut id = status_id.clone();
    let mut status = model.single_status(id.clone()).await?;
    let mut conversation = model.status_context(id).await?;
    if !conversation.ancestors.is_empty() {
        status = conversation.ancestors[0].clone();
        id = status.id.clone();
        conversation = model.status_context(id).await?;
    }

    use id_tree::InsertBehavior::*;

    let mut tree: Tree<StatusViewModel> = TreeBuilder::new().with_node_capacity(32).build();

    let root_id: NodeId = tree
        .insert(Node::new(StatusViewModel::new(&status)), AsRoot)
        .map_err(convert)?;

    // keep the node-ids to status-ids so we can correctly insert
    let mut ids = HashMap::new();
    ids.insert(&status.id, root_id);

    for status in conversation.descendants.iter() {
        let Some(reply_id) = status.in_reply_to_id.as_ref().and_then(|id| ids.get(&id)) else {
            log::error!("Could not resolve reply-to for status {}", status.id);
            continue
        };
        let Ok(child_id) = tree.insert(
            Node::new(StatusViewModel::new(status)),
            UnderNode(reply_id),
        ) else {
            log::error!("Could not insert status into tree {}", status.id);
            continue
        };
        ids.insert(&status.id, child_id.clone());
    }

    let conv = Conversation {
        original_status: status_id,
        tree,
    };

    Ok(conv)
}

fn convert(value: NodeIdError) -> String {
    format!("{value:?}")
}
