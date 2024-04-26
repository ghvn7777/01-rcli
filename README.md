# RCLI

rcli is a rust CLI tool.

## 一、Csv convert

### 1.1 csv 转 json
```
cargo run csv --input assets/juventus.csv # 默认 format 是 json
```

### 1.2 csv 转 yaml
```
cargo run csv --input assets/juventus.csv --format yaml
```

还有更多参数，指定分隔符，指定输出文件名等等

## 二、生成密码

可以选择大小写、特殊字符、长度等参数，默认的就挺好
```
cargo run -- genpass
```

## 三、base64 编码/解码

### 3.1 base64 encode
```
cargo run -- base64 encode
```

### 3.2 base64 decode
```
cargo run -- base64 decode
```

可以指定文件以及转换格式，默认是标准转换，可以选择 format 为 urlsafe

## 四、sign and verify

### 4.1 blake3 对称加密算法进行签名

生成 key
```
cargo run text generate --output-path fixtures/ # 默认模式是 blake3
```

进行加密
```
cargo run text sign -k fixtures/blake3.txt
# 然后输入消息，例如 hello，不要按回车，直接 ctrl + d
# 会得到一个 sig，例如 sig 是 Qdw8wZnrwWWo7Dvle47_A4iLT39fdHxh5xsh2JLQAyE
```

验证加密结果
```
cargo run text verify -k fixtures/blake3.txt --sig Qdw8wZnrwWWo7Dvle47_A4iLT39fdHxh5xsh2JLQAyE
# 再次输入 hello 然后 Ctrl+D 可以看到验证成功
```

### 4.2 ed25519 非对称加密算法进行签名

生成公钥私钥
```
cargo run text generate --output-path fixtures/ --format ed25519 # 生成 sk, pk 文件
```

使用私钥进行签名
```
cargo run text sign -k fixtures/ed25519_sk.txt --format ed25519
# 然后输入消息，例如 hello，不要按回车，直接 ctrl + d
# 会得到一个 sig，例如 sig 是 Zbii_ujaR6KClq5wXF_yg3fHgi-5Dr0_CLtEzz_Sso9sgNG23385xd3xcB1s-LF5QJ7IbHt7OKZuEHe1Pt6JCw
```

使用公钥进行验证
```
cargo run text verify -k fixtures/ed25519_pk.txt --format ed25519 --sig Zbii_ujaR6KClq5wXF_yg3fHgi-5Dr0_CLtEzz_Sso9sgNG23385xd3xcB1s-LF5QJ7IbHt7OKZuEHe1Pt6JCw
# 再次输入 hello 然后 Ctrl+D 可以看到验证成功
```

### 4.3 chacha20poly 算法对文本加密解密

**生成密钥**
```
cargo run -- text generate --output-path fixtures/ --format chacha20
```

会生成 `fixtures/chacha20.txt` 文件

**加密文本**
```
cargo run -- text encrypt --key fixtures/chacha20.txt --format chacha20
```
然后输入要加密的文本，ctrl + D 结束输入，也可以用 `-i` 参数指定要加密的文件

这里我们会得到一段 base64 编码的文本，可以复制下来，一会解密用

如果想直接把 base64 编码文本保存到文件中，可以用 --output 选项

**解密文本**
```
cargo run -- text decrypt --key fixtures/chacha20.txt --format chacha20
```

然后输入上面的 base64 编码文本，就可以看到解密的结果了。也可以使用 `-i` 参数指定要解密的文件

### 4.4 jwt 签名验证

**生成 jwt secret**
```
cargo run jwt generate --output-path fixtures/
```
会生成 `fixtures/jwt.txt` 文件

**jwt 签名**
```
cargo run -- jwt sign --key fixtures/jwt.txt --sub "rust work" --aud "kk" --exp 24h --algorithm HS512
```
这里算法可以不指定，默认是 `HS256`，该命令运行后会得到 token

**token 签名验证**
```
cargo run -- jwt verify --key fixtures/jwt.txt --token "上面得到的结果"
```
现在代码里面把所有字段验证都关了，只验证签名，其实可以对每个字段、过期时间等等进行验证


## 五、静态服务器
```
RUST_LOG=info cargo run -- http serve
```
