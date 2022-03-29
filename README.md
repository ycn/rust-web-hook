# web hook service

1. 只做好一件事: 记录有效请求到日志
2. 有效请求: `token=hash(ts+key)`
3. 日志 scoped: 根据请求路径自动生成文件
4. 标准化 RequestData: `{ type: '类型', from: '来源', data: '数据' }`
5. 自动为请求数据生成 uuid, 可检索和追踪 //TODO
6. 支持写队列 //TODO
