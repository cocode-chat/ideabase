CREATE DATABASE `ideabase` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin */ /*!80016 DEFAULT ENCRYPTION='N' */

CREATE TABLE account (
    id                 BIGINT PRIMARY KEY COMMENT '用户唯一ID',
    email              VARCHAR(255) NOT NULL COMMENT '用户邮箱地址',
    phone              VARCHAR(50)  NOT NULL COMMENT '用户手机号码',
    password           VARCHAR(128) NOT NULL COMMENT '用户密码(加密存储)',
    role               VARCHAR(50)  NOT NULL COMMENT '用户角色(admin/user等)',
    api_key            VARCHAR(128) NULL COMMENT '用户API密钥',
    email_confirmed_at DATETIME NULL COMMENT '邮箱确认时间',
    last_sign_in_at    DATETIME NULL COMMENT '最后登录时间',
    gmt_create         DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '记录创建时间',
    gmt_update         DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '记录更新时间',
    INDEX              idx_email (email),
    INDEX              idx_phone (phone)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='账户信息表';