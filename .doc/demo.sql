CREATE DATABASE `timeline` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_bin */ /*!80016 DEFAULT ENCRYPTION='N' */

create table if not exists comment (
    id                bigint                             not null comment '评论唯一标识' primary key,
    moment_id         bigint                             not null comment '所属动态 ID',
    user_id           bigint                             not null comment '评论用户 ID',
    parent_comment_id int                                null comment '父评论ID（如果是回复评论则不为空）',
    content           text                               not null comment '评论内容',
    like_count        int      default 0                 null comment '评论点赞数',
    gmt_create        datetime default CURRENT_TIMESTAMP null comment '评论创建时间',
    gmt_update        datetime default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '评论更新时间'
) comment '评论表' charset = utf8mb4;

create index idx_moment on comment (moment_id, user_id);

create table if not exists moment (
    id         bigint                             not null comment '动态唯一标识' primary key,
    user_id    bigint                             not null comment '发布动态的用户 ID',
    content    text                               null comment '动态内容',
    image_urls json                               null comment '图片 URL 列表，存储为 JSON 格式',
    location   varchar(200)                       null comment '发布位置',
    like_cnt   int      default 0                 null comment '点赞数',
    comment_ct int      default 0                 null comment '评论数',
    gmt_create datetime default CURRENT_TIMESTAMP null comment '动态创建时间',
    gmt_update datetime default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '动态更新时间'
) comment '动态表' charset = utf8mb4;

create index idx_user on moment (user_id);

create table if not exists user (
    id         bigint                             not null comment '用户唯一标识'  primary key,
    username   varchar(64)                        not null comment '用户名',
    nickname   varchar(64)                        not null comment '用户昵称',
    avatar     varchar(200)                       null comment '用户头像 URL',
    gender     enum ('male', 'female', 'other')   null comment '性别',
    phone      varchar(20)                        null comment '手机号',
    email      varchar(100)                       null comment '电子邮箱',
    gmt_create datetime default CURRENT_TIMESTAMP null comment '用户创建时间',
    gmt_update datetime default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment '用户信息更新时间'
) comment '用户表' charset = utf8mb4;

