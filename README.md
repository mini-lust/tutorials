# Mini Lust 系列教程

好奇如何从零造出来一个 RPC 框架？本教程将带你一步一步写出来一个 Rust 版 Thrift RPC 框架。

## 教程说明
从第二章开始每章节都会附带代码。
这个代码是在上一章节的基础上写的，文档里一般会告诉你增加了哪些东西，但是如果你想详细地对比到底哪里变动了，可以自行 diff。

每章的代码会尽量通过测试保证代码是正确工作的，我们会通过 CI 来向你保证这一点。


## 教程结构
依次分几个章节：
1. 前言部分，RPC 相关概念介绍
2. Thrift IDL 介绍
3. 序列化/反序列化的抽象
4. Codec 和 Transport 抽象
5. 客户端和服务端实现
6. Thrift IDL 解析和代码生成
7. 基于 tower 的服务发现和负载均衡
8. 中间件支持

## Credit
- [@dyxushuai](https://github.com/dyxushuai): Lust project creator and helped mini-lust project a lot
- [@ihciah](https://github.com/ihciah): Mini-lust tutorial creator and Lust project developer
- [@PureWhiteWu](https://github.com/PureWhiteWu): Lust project developer
- [@LYF1999](https://github.com/LYF1999): Lust project developer
- [@Millione](https://github.com/Millione): Lust project developer

## 进度
- 1～6 章节代码基本写完，教程尚需优化，部分细节需优化
- CI 配置待补齐
