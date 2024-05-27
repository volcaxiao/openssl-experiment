# XSS

## XSS 攻击

XSS（Cross-Site Scripting）跨站脚本攻击, 为不和层叠样式表（Cascading Style Sheets, CSS）的缩写混淆, 故将跨站脚本攻击缩写为XSS。

XSS是一种代码注入攻击。攻击者通过在目标网站上注入恶意代码, 使之在用户的浏览器上运行。利用这些恶意代码, 攻击者可获取用户的敏感信息如: Cookie、SessionID 等, 进而危害数据安全。

## XSS 攻击原理

XSS 攻击原理: 

- 攻击者将恶意代码注入到目标网站上, 当用户浏览该网站时, 恶意代码会自动执行, 从而达到攻击者的目的。
- 恶意代码通常以 JavaScript 代码的形式存在, 但也可以是 Java、VBScript、ActiveX 等其他形式。
- 恶意代码可以包含在网页的 HTML 代码中, 也可以是来自其他网站的图像、视频等资源。
- 恶意代码可以隐藏在正常内容中, 很难被检测出来。

## XSS 攻击防御

XSS 攻击的防御是针对攻击者注入的恶意代码进行过滤、转义和校正。
- 转义: 对特殊字符进行转义, 如 `<` 转换为 `&lt;`, `>` 转换为 `&gt;`。
- 校正: 对输入进行校正, 如过滤掉 HTML 标签。
- 内容安全策略(Content Security Policy, CSP): 通过设置 HTTP 头部 `Content-Security-Policy` 来限制页面的加载和执行的资源, 从而防止 XSS 攻击。

## 实现
### Get
- 设置内容安全策略
	- `/XSS/safe`
- 未设置内容安全策略
	- `/XSS/unsafe`

### Post
- 过滤输入
	- `/XSS/safe`
- 未过滤输入
	- `/XSS/unsafe`

