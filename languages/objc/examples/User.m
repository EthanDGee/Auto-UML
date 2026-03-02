#import <Foundation/Foundation.h>

@interface User : NSObject
@property (nonatomic, assign) long long userId;
@property (nonatomic, copy) NSString *username;
@property (nonatomic, copy) NSString *email;
@end

@implementation User
@end
