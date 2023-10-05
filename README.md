# wlt_task

## 命令及作用

```
wlt_task             打开命令行交互界面
wlt_task run         登录WLT并在IP变化时发送邮件
wlt_task set         将wlt_task设置为每5分钟+网络连接时运行的计划任务（仅Windows可用）
wlt_task unset       取消计划任务（仅Windows可用）
wlt_task query       查询计划任务的状态（仅Windows可用）
```

## 使用说明

### Windows

将`wlt_task.exe`放在一个固定的文件夹里，其产生的文件都会在这个文件夹。

双击`wlt_task.exe`，选择1执行，产生`config.toml`文件。

根据`config.toml`中的提示填写该文件，注意邮箱密码一般都是SMTP授权码。

双击`wlt_task.exe`，选择2执行，设置计划任务。

双击`wlt_task.exe`，选择4执行，查看计划任务相关信息。

## 日志说明

`log.txt`中的内容为日志，日志中的`.`表示脚本成功执行了一次，`?`表示一次访问超时，其余行包含日期时间和信息，一个示例如下（`*`号处为不便展示的内容）：

```
2023-09-26 23:38:09: 输入的用户名为空
2023-09-26 23:38:09: 没有设置"邮件发送列表"，不发送邮件
2023-09-26 23:42:00: 旧rn:  新rn: *
2023-09-26 23:42:00: 旧IP:  新IP: 114.*.*.*
2023-09-26 23:42:00: 没有设置"邮件发送列表"，不发送邮件
......
```

## 卸载说明

### Windows

双击`wlt_task.exe`，选择3执行，若没有报错，则取消计划任务成功。之后删除文件夹即可。

## TODO

- `set`, `unset`, `query`命令支持Linux/macOS

*star + issue = 火速更新*
