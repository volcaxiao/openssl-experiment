mkdir -p ./SSL/Cert
cd ./SSL/Cert || (exit; echo "no such directory: ./SSL/Cert")
# 1、生成RSA私钥文件
echo "1、生成RSA私钥文件"
openssl genrsa -des3 -out ca.key 2048
# 2、创建证书签名请求文件
echo "2、创建证书签名请求文件"
openssl req -new -key ca.key -out ca.csr
# 3、使用私钥为证书签名（自签名）
echo "3、使用私钥为证书签名（自签名）"
openssl x509 -req -extensions v3_ca -days 3650 -in ca.csr -signkey  ca.key -out ca.crt
# 4. 生成ca.pem（用于导出，导入证书时候的证书的格式），并将此证书导入电脑受信任根证书目录下
openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 -out ca_cert.pem
echo "请将ca.crt证书导入电脑受信任根证书目录下"
cd ../../
