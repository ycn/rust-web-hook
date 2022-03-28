# web hook service

1. 只做好一件事: 记录有效请求到日志(单向)
2. 有效请求: token=hash(ts+key)
3. 日志scoped: 根据请求路径生成文件
4. 标准化RequestData: { cat: '类型', from: '来源', data: '数据' }
5. 自动为请求数据生成uuid, 可检索和追踪
6. 标准化Response: 200{ id: 'uuid' }, 404{}
7. 支持写队列
