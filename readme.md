# 1. 监听端口, 输出 index.html                                   (ok)
# 2. 创建 Request(GET), 并把浏览器的请求转换成 Request(GET)        (ok)
# 3. 数据帧- 每次读10个到缓存buff中, buff满了就开始对buff解析,
# 特定解析完成后移动游标或删除已解析buff                            (待定)
# 4. 写个缓存buff                                               (ok)
# TODO 写路由宏

# TODO client 应当持有一个 缓存区, 这样就不需要总是创建新的 Bytes