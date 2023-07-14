window.addEventListener("load", (event) => {
    window.addEventListener('blur', (event) => {
        const instances = document.querySelector(".login-instance.selected")
        if (instances != null) {
            instances.classList.add("inactive");
        }
        const sidebar = document.querySelector(".cell.selected");
        if (sidebar != null) {
            sidebar.classList.add("inactive");
        }
        const cell = document.querySelector(".content-cell.cell-selected")
        if (cell != null) {
            cell.classList.add("inactive");
        }
        const ccell = document.querySelector(".conversation-child-selected")
        if (ccell != null) {
            ccell.classList.add("inactive");
        }
    });
    window.addEventListener('focus', (event) => {
        const instances = document.querySelector(".login-instance.selected")
        if (instances != null) {
            instances.classList.remove("inactive");
        }
        const sidebar = document.querySelector(".cell.selected");
        if (sidebar != null) {
            sidebar.classList.remove("inactive");
        }
        const cell = document.querySelector(".content-cell.cell-selected")
        if (cell != null) {
            cell.classList.remove("inactive");
        }
        const ccell = document.querySelector(".conversation-child-selected")
        if (ccell != null) {
            ccell.classList.remove("inactive");
        }
    });
});

function createMenu(items, x, y) {
    console.log(items);
    // Create a click catcher for closing the menu
    const catcher = document.createElement("div");
    catcher.classList.add("menu-catcher");
    catcher.onclick = () => {
        menu.remove();
        catcher.remove();
        window.catcher = null;
    };
    document.body.appendChild(catcher);
    window.catcher = catcher;


    const menu = document.createElement("div");
    menu.classList.add("context-menu");
    buildMenuItems(menu, items, (id) => {
        menu.remove();
        catcher.remove();
        window.catcher = null;
        const msg =
            JSON.stringify({
                "method": "user_event",
                "params": {
                    "name": "action",
                    "data": id,
                    "bubbles": true,
                }
            });
        console.log(msg);
        window.ipc.postMessage(
            msg
        );
    });
    menu.style.left = x + "px";
    menu.style.top = y + "px";
    catcher.appendChild(menu);
}

function buildMenuItems(container, items, clicked) {
    const menu = document.createElement("ul");
    for (const item of items) {
        const menuItem = document.createElement("li");
        menuItem.classList.add("context-menu-item");
        menuItem.innerText = item.title;
        if (item.kind == "checkbox") {
            menuItem.classList.add("context-menu-entry");
            const checkbox = document.createElement("input");
            checkbox.type = "checkbox";
            checkbox.checked = item.checked;
            checkbox.disabled = !item.enabled;
            checkbox.onclick = () => {
                clicked(item.id);
            };
            menuItem.insertBefore(checkbox, menuItem.firstChild);
        } else if (item.kind == "submenu") {
            menuItem.classList.add("context-menu-menu");
            buildMenuItems(menuItem, item.submenu, clicked);
        } else if (item.kind == "separator") {
            menuItem.classList.add("context-menu-separator");
            menuItem.innerText = "";
        } else if (item.kind == "menuitem") {
            menuItem.classList.add("context-menu-entry");
            menuItem.classList.add("context-menu-highlight");
            if (item.enabled) {
                menuItem.style.cursor = "pointer";
                menuItem.onclick = () => {
                    clicked(item.id);
                };
            }
        }

        menu.appendChild(menuItem);
    }
    container.appendChild(menu);
}