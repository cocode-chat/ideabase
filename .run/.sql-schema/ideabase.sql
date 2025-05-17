CREATE
DATABASE `ideabase` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin */ /*!80016 DEFAULT ENCRYPTION='N' */

CREATE TABLE account (
    id                 bigint                             not null comment '用户唯一ID' primary key,
    email              varchar(255)                       not null comment '用户邮箱地址',
    password           varchar(128)                       not null comment '用户密码(加密存储)',
    role               varchar(32)                        not null comment '用户角色(admin/user等)',
    api_key            varchar(128) null comment '用户API密钥',
    email_confirmed_at datetime null comment '邮箱确认时间',
    last_sign_in_at    datetime null comment '最后登录时间',
    gmt_create         datetime default CURRENT_TIMESTAMP not null comment '记录创建时间',
    gmt_update         datetime default CURRENT_TIMESTAMP not null on update CURRENT_TIMESTAMP comment '记录更新时间',
    constraint idx_email unique (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='账户信息表';