#include "bindings/bindings.h"
#include <UIKit/UIKit.h>

int main(int argc, char * argv[]) {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.0 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
        NSLog(@"run after");
        UIWindow *window = [[UIApplication sharedApplication] keyWindow];
        UIViewController *vc = [window rootViewController];
        
        // we only have one subview
        UIView *subview = vc.view.subviews.firstObject;
        [subview removeConstraints:subview.constraints];
        
        CGFloat bottomPadding = window.safeAreaInsets.bottom;
        
        CGFloat bottomPadding2 = subview.safeAreaInsets.bottom;
        
        [subview setTranslatesAutoresizingMaskIntoConstraints:false];
        
        [NSLayoutConstraint activateConstraints:@[
            [subview.leadingAnchor constraintEqualToAnchor:vc.view.leadingAnchor],
            [subview.trailingAnchor constraintEqualToAnchor:vc.view.trailingAnchor],
            [subview.topAnchor constraintEqualToAnchor:vc.view.topAnchor],
            [subview.bottomAnchor constraintEqualToAnchor:window.bottomAnchor constant:bottomPadding + bottomPadding2],
        ]];
    });
    run();
	return 0;
}
