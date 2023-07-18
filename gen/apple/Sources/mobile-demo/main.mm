#include "bindings/bindings.h"
#include <UIKit/UIKit.h>

int main(int argc, char * argv[]) {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(5.0 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
        NSLog(@"run after");
        UIViewController *vc = [[[UIApplication sharedApplication] keyWindow] rootViewController];
        vc.view.insetsLayoutMarginsFromSafeArea = NO;
        vc.view.layoutMargins = UIEdgeInsets();
        for (UIView *subview in vc.view.subviews) {
            subview.insetsLayoutMarginsFromSafeArea = NO;
            subview.layoutMargins = UIEdgeInsets();
        }
        NSLog(@"VC: %@", vc);
    });
    run();
	return 0;
}
