#import <Foundation/Foundation.h>

@interface Box : NSObject
@property (nonatomic, strong) id inner;
- (instancetype)initWithInner:(id)inner;
- (id)get;
@end

@implementation Box
- (instancetype)initWithInner:(id)inner {
    self = [super init];
    if (self) {
        _inner = inner;
    }
    return self;
}
- (id)get {
    return self.inner;
}
@end
