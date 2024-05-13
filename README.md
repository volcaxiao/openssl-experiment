# openssl-experiment

小组在中期使用Rust语言实现了一版简单的server与client。
其中，server能够支持：
1. 加载 SSL 证书和私钥。
2. 监听特定的端口。
3. 处理Get请求。（其他请求暂时返回404）

client能够支持：
1. 发送Get请求。
2. 

## 功能测试

- 

## 实验分析

- 

## 后期计划

- 继续实现Server Client的其他功能。
- 完成跨站脚本攻击（XSS），会话劫持，DOS攻击的攻防演练。

## 小组分工

- 罗皓天：实现Server程序。
- 戴波：实现Client程序。
- 肖灿：撰写文档，实验分析，配置openssl证书与链接。