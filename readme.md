# 1. 监听端口, 输出 index.html (ok)

# 2. 创建 Request(GET), 并把浏览器的请求转换成 Request(GET) (ok)

# 3. 数据帧- 每次读 10 个到缓存 buff 中, buff 满了就开始对 buff 解析,

# 特定解析完成后移动游标或删除已解析 buff (待定)

# 4. 写个缓存 buff (ok)

# 5. 写路由宏 (ok)

# 6. 整理 + 写 error.rs (ok)

# 7. client 应当持有一个 缓存区, 这样就不需要总是创建新的 Bytes (ok)

# 8. 解析请求出错了咋办; run 函数中途或 func 函数中途对方断开连接咋办 (ok)

# TODO a. 研究 listener.poll_accept() 以尝试 join! 多个连接.
# b. 研究 futures 的 join_all 和 MaybeDone, 看看别人是怎么防止重复 `poll` 的 
# TODO c. 用 join 技术来开发 send_file 方法