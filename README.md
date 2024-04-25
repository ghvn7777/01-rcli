# RCLI

rcli is a rust CLI tool.

## Csv convert

csv 转 json
```
cargo run csv --input assets/juventus.csv # 加上 --format json
```

csv 转 yaml
```
cargo run csv --input assets/juventus.csv --format yaml
```

还有更多用法，定分隔符，输出文件名等等

## 生成密码

可以选择大小写什么的
```
cargo run -- genpass
```

## base64 转换
转换命令行输入
```
cargo run -- base64 encode
cargo run -- base64 decode
```

可以指定文件以及转换格式，默认是标准转换，可以选择 format 为 urlsafe

## sign and verify

对称加密
```
cargo run text generate --output-path fixtures/ # 默认模式是 blake3
cargo run text sign -k fixtures/blake3.txt
# 然后输入消息，例如 hello，不要按回车，直接 ctrl + d
# 会得到一个 sig，例如 sig 是 Qdw8wZnrwWWo7Dvle47_A4iLT39fdHxh5xsh2JLQAyE
cargo run text verify -k fixtures/blake3.txt --sig Qdw8wZnrwWWo7Dvle47_A4iLT39fdHxh5xsh2JLQAyE
# 再次输入 hello 然后 Ctrl+D 可以看到验证成功
```

非对称加密
```
cargo run text generate --output-path fixtures/ --format ed25519 # 生成 sk, pk 文件
cargo run text sign -k fixtures/ed25519_sk.txt --format ed25519
# 然后输入消息，例如 hello，不要按回车，直接 ctrl + d
# 会得到一个 sig，例如 sig 是 Zbii_ujaR6KClq5wXF_yg3fHgi-5Dr0_CLtEzz_Sso9sgNG23385xd3xcB1s-LF5QJ7IbHt7OKZuEHe1Pt6JCw
# cargo run text verify -k fixtures/ed25519_pk.txt --format ed25519 --sig Zbii_ujaR6KClq5wXF_yg3fHgi-5Dr0_CLtEzz_Sso9sgNG23385xd3xcB1s-LF5QJ7IbHt7OKZuEHe1Pt6JCw
# 再次输入 hello 然后 Ctrl+D 可以看到验证成功
```

## text encrypt and decrypt
生成密钥
```
cargo run -- text generate --output-path fixtures/ --format chacha20
```

会生成 `fixtures/chacha20.txt` 文件

## 文本加密
```
cargo run -- text encrypt --key fixtures/chacha20.txt --format chacha20
```
然后输入要加密的文本，ctrl + D 结束输入，也可以用 `-i` 参数指定要加密的文件

这里我们会得到一段 base64 编码的文本，可以复制下来或者存到文件中，一会解密用

## 文本解密
```
cargo run -- text decrypt --key fixtures/chacha20.txt --format chacha20
```

然后输入上面的 base64 编码文本，就可以看到解密的结果了。也可以使用 `-i` 参数指定要解密的文件
