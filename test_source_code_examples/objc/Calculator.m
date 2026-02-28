#import <Foundation/Foundation.h>

@interface Calculator : NSObject
- (int)add:(int)a b:(int)b;
- (void)clear;
@end

@implementation Calculator
- (int)add:(int)a b:(int)b {
    return a + b;
}
- (void)clear {
    // ...
}
@end
