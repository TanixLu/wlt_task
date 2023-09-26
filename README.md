# wlt_task

## 命令及作用

```
Windows、Linux、macOS都支持的命令：
wlt_task             登录WLT并在IP变化时发送邮件
仅Windows支持的命令：
wlt_task set         将wlt_task设置为每5分钟运行一次的计划任务
wlt_task unset       取消计划任务
wlt_task query       查询计划任务的状态
```

## 使用说明

### Windows

将`wlt_task.exe`放在一个固定的文件夹里，其产生的文件都会在这个文件夹。

先点击运行一次`wlt_task.exe`，会产生`config.toml`和`log.txt`两个文件。

根据`config.toml`中的提示填写该文件，注意邮箱密码一般都是SMTP授权码。

在地址栏中输入`cmd`后回车，打开命令行，输入`wlt_task set`并回车运行，其会生成一个`wlt_task.vbs`文件，若未报错，则设置计划任务成功。

可以通过`wlt_task query`查看计划任务相关信息。

## 日志说明

`log.txt`中的内容为日志，日志中的`.`表示脚本成功执行了一次，其余行包含日期时间和信息，一个示例如下（`*`号处为不便展示的内容）：

```
2023-09-26 23:38:09: 输入的用户名为空
2023-09-26 23:38:09: 没有设置email_to_list，不发送邮件
2023-09-26 23:42:00: old_rn:  new_rn: *
2023-09-26 23:42:00: old_ip:  new_ip: 114.*.*.*
2023-09-26 23:42:00: 没有设置email_to_list，不发送邮件
......
```

## 卸载说明

### Windows

在`wlt_task.exe`文件夹打开`cmd`，输入`wlt_task unset`并回车运行，若没有报错，则说明取消计划任务成功。

之后删除同一文件夹下的`wlt_task.exe`、`config.toml`、`log.txt`、`wlt_task.vbs`文件，这样就删除了所有`wlt_task`组件。

## TODO

- `set`, `unset`, `query`命令支持Linux/macOS
- windows系统上，将命令行指令改为弹出窗口并给出选项来选择，方便用户使用；增加卸载命令

*star + issue = 火速更新*
