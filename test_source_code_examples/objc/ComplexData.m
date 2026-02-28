#import <Foundation/Foundation.h>

@class User;

@interface Metadata : NSObject
@property (nonatomic, copy) NSString *key;
@property (nonatomic, copy) NSString *value;
- (void)log;
@end

@implementation Metadata
- (void)log {}
@end

@interface ComplexData : NSObject
@property (nonatomic, strong) NSData *raw_bytes;
@property (nonatomic, strong) NSArray<NSString *> *items;
@property (nonatomic, strong) NSDictionary<NSString *, NSNumber *> *config;
@property (nonatomic, strong) User *owner;

- (void)process:(NSString *)mode;
+ (ComplexData *)create;
@end

@implementation ComplexData
- (void)process:(NSString *)mode {
    // ...
}
+ (ComplexData *)create {
    return [[ComplexData alloc] init];
}
@end
