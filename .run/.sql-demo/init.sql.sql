create DATABASE `ecommerce` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci */ /*!80016 DEFAULT ENCRYPTION='N' */;
USE `ecommerce`;

--
-- Table structure for table `item`
--

DROP TABLE IF EXISTS `item`;
CREATE TABLE `item` (
    `id`             int                                                           NOT NULL AUTO_INCREMENT COMMENT '商品ID',
    `name`           varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '商品名',
    `description`    varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '商品描述',
    `category`       varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '商品类别',
    `price`          decimal(10, 2)                                                NOT NULL COMMENT '商品价格',
    `stock_quantity` int                                                           NOT NULL COMMENT '库存数量',
    `gmt_create`     timestamp NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    PRIMARY KEY (`id`),
    CONSTRAINT `item_chk_1` CHECK ((`price` >= 0)),
    CONSTRAINT `item_chk_2` CHECK ((`stock_quantity` >= 0))
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='商品';

--
-- Dumping data for table `item`
--

INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (1, '1克拉钻石项链', '经典18K白金镶嵌1克拉钻石项链，闪耀夺目', '钻石首饰', 8999.00, 15, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (2, '0.5克拉钻石戒指', '18K玫瑰金镶嵌0.5克拉钻石戒指，浪漫典雅', '钻石首饰', 4599.00, 22, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (3, '钻石耳钉套装', '18K黄金镶嵌0.3克拉钻石耳钉，精致小巧', '钻石首饰', 2999.00, 35, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (4, '钻石吊坠', '18K白金镶嵌0.2克拉钻石吊坠，简约时尚', '钻石首饰', 1899.00, 40, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (5, '钻石手链', '18K玫瑰金镶嵌多颗小钻石手链，优雅大方', '钻石首饰', 3299.00, 18, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (6, '豪华钻石戒指', '铂金镶嵌2克拉钻石戒指，奢华闪耀', '钻石首饰', 18999.00, 5, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (7, '钻石婚戒套装', '18K白金镶嵌1.2克拉钻石婚戒套装，见证永恒爱情', '钻石首饰', 12999.00, 12,
        '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (8, '钻石胸针', '18K黄金镶嵌0.4克拉钻石胸针，彰显高贵气质', '钻石首饰', 5699.00, 8, '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (9, '钻石情侣对戒', '18K玫瑰金镶嵌0.3克拉钻石情侣对戒，浪漫情侣必备', '钻石首饰', 5299.00, 25,
        '2025-05-09 02:36:47');
INSERT INTO `item` (`id`, `name`, `description`, `category`, `price`, `stock_quantity`, `gmt_create`)
VALUES (10, '钻石镶嵌腕表', '奢华钻石镶嵌女士腕表，表盘点缀多颗小钻石', '钻石首饰', 15999.00, 7, '2025-05-09 02:36:47');

--
-- Table structure for table `order`
--

DROP TABLE IF EXISTS `order`; CREATE TABLE `order` (
    `id`               int                                     NOT NULL AUTO_INCREMENT COMMENT '订单ID',
    `customer_id`      int                                     NOT NULL COMMENT '用户ID',
    `total_amount`     decimal(10, 2)                          NOT NULL COMMENT '订单金额',
    `status`           varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '订单状态',
    `order_date`       datetime                                NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '下单时间',
    `shipping_address` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '邮寄地址',
    PRIMARY KEY (`id`),
    CONSTRAINT `order_chk_1` CHECK ((`status` in
                                     (_utf8mb4'pending', _utf8mb4'processing', _utf8mb4'shipped', _utf8mb4'delivered',
                                      _utf8mb4'cancelled'))),
    CONSTRAINT `order_chk_2` CHECK ((`total_amount` >= 0))
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='订单';

--
-- Dumping data for table `order`
--

INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (1, 1001, 13898.00, 'shipped', '2024-10-01 10:30:45', '北京市朝阳区钻石大厦A座');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (2, 1002, 6498.00, 'processing', '2024-10-02 14:22:10', '上海市浦东新区陆家嘴环路1000号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (3, 1003, 20997.00, 'delivered', '2024-10-03 09:15:30', '广州市天河区珠江新城花城大道88号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (4, 1004, 9897.00, 'pending', '2024-10-04 16:45:20', '深圳市南山区科技园科发路8号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (5, 1005, 4499.00, 'shipped', '2024-10-05 11:05:15', '成都市高新区天府大道北段1700号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (6, 1006, 16899.00, 'processing', '2024-10-06 18:30:05', '杭州市西湖区文三路90号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (7, 1007, 3798.00, 'cancelled', '2024-10-07 08:55:40', '重庆市渝中区解放碑步行街100号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (8, 1008, 24798.00, 'delivered', '2024-10-08 13:12:25', '南京市鼓楼区汉中路140号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (9, 1009, 7598.00, 'pending', '2024-10-10 15:40:35', '武汉市武昌区中北路181号');
INSERT INTO `order` (`id`, `customer_id`, `total_amount`, `status`, `order_date`, `shipping_address`)
VALUES (10, 1010, 19998.00, 'shipped', '2024-10-10 17:20:55', '西安市雁塔区科技路101号');

--
-- Table structure for table `order_items`
--

DROP TABLE IF EXISTS `order_items`;
CREATE TABLE `order_items` (
    `id`             int            NOT NULL AUTO_INCREMENT,
    `order_id`       int            NOT NULL,
    `item_id`        int            NOT NULL,
    `quantity`       int            NOT NULL,
    `price_per_item` decimal(10, 2) NOT NULL,
    PRIMARY KEY (`id`),
    KEY              `idx_order_id` (`order_id`),
    KEY              `idx_item_id` (`item_id`),
    CONSTRAINT `order_items_ibfk_1` FOREIGN KEY (`order_id`) REFERENCES `order` (`id`) ON DELETE CASCADE,
    CONSTRAINT `order_items_ibfk_2` FOREIGN KEY (`item_id`) REFERENCES `item` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `order_items_chk_1` CHECK ((`quantity` > 0)),
    CONSTRAINT `order_items_chk_2` CHECK ((`price_per_item` >= 0))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci comment "订单明细";

--
-- Dumping data for table `order_items`
--

INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (1, 1, 1, 1, 8999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (2, 1, 5, 1, 3299.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (3, 2, 2, 1, 4599.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (4, 2, 3, 1, 2999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (5, 3, 6, 1, 18999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (6, 3, 4, 1, 1899.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (7, 4, 7, 1, 12999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (8, 4, 8, 1, 5699.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (9, 5, 9, 1, 5299.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (10, 5, 10, 1, 15999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (11, 6, 3, 1, 2999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (12, 6, 6, 1, 18999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (13, 7, 10, 1, 15999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (14, 7, 2, 1, 4599.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (15, 8, 7, 1, 12999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (16, 8, 1, 1, 8999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (17, 9, 4, 1, 1899.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (18, 9, 9, 1, 5299.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (19, 10, 6, 1, 18999.00);
INSERT INTO `order_items` (`id`, `order_id`, `item_id`, `quantity`, `price_per_item`)
VALUES (20, 10, 5, 1, 3299.00);


CREATE DATABASE `timeline` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci */ /*!80016 DEFAULT ENCRYPTION='N' */;
USE `timeline`;
--
-- Table structure for table `comment`
--

DROP TABLE IF EXISTS `comment`;
CREATE TABLE `comment` (
                           `id` bigint NOT NULL COMMENT '评论唯一标识',
                           `moment_id` bigint NOT NULL COMMENT '所属动态 ID',
                           `user_id` bigint NOT NULL COMMENT '评论用户 ID',
                           `parent_comment_id` int DEFAULT NULL COMMENT '父评论ID（如果是回复评论则不为空）',
                           `content` text NOT NULL COMMENT '评论内容',
                           `like_count` int DEFAULT '0' COMMENT '评论点赞数',
                           `gmt_create` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '评论创建时间',
                           `gmt_update` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '评论更新时间',
                           PRIMARY KEY (`id`),
                           KEY `idx_moment` (`moment_id`,`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='评论表';

--
-- Dumping data for table `comment`
--

INSERT INTO `comment` (`id`, `moment_id`, `user_id`, `parent_comment_id`, `content`, `like_count`, `gmt_create`, `gmt_update`) VALUES (18710,28710,38710,0,'咔咔咔咔',0,'2025-05-06 12:00:31','2025-05-06 12:00:31');
INSERT INTO `comment` (`id`, `moment_id`, `user_id`, `parent_comment_id`, `content`, `like_count`, `gmt_create`, `gmt_update`) VALUES (18711,28711,38710,0,'呼呼呼呼',0,'2025-05-06 12:03:12','2025-05-06 12:03:12');

--
-- Table structure for table `moment`
--

DROP TABLE IF EXISTS `moment`;
CREATE TABLE `moment` (
                          `id` bigint NOT NULL COMMENT '动态唯一标识',
                          `user_id` bigint NOT NULL COMMENT '发布动态的用户 ID',
                          `content` text COMMENT '动态内容',
                          `image_urls` json DEFAULT NULL COMMENT '图片 URL 列表，存储为 JSON 格式',
                          `location` varchar(200) DEFAULT NULL COMMENT '发布位置',
                          `like_cnt` int DEFAULT '0' COMMENT '点赞数',
                          `comment_ct` int DEFAULT '0' COMMENT '评论数',
                          `gmt_create` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '动态创建时间',
                          `gmt_update` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '动态更新时间',
                          PRIMARY KEY (`id`),
                          KEY `idx_user` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='动态表';

--
-- Dumping data for table `moment`
--

INSERT INTO `moment` (`id`, `user_id`, `content`, `image_urls`, `location`, `like_cnt`, `comment_ct`, `gmt_create`, `gmt_update`) VALUES (28710,38710,'第一条朋友圈','[\"moment/111111.png\", \"moment/111112.png\"]',NULL,0,0,'2025-05-06 11:44:55','2025-05-06 12:00:03');
INSERT INTO `moment` (`id`, `user_id`, `content`, `image_urls`, `location`, `like_cnt`, `comment_ct`, `gmt_create`, `gmt_update`) VALUES (28711,38711,'this is a show!','[\"moment/111111.png\", \"moment/111112.png\"]',NULL,0,0,'2025-05-06 11:58:13','2025-05-06 12:00:03');

--
-- Table structure for table `user`
--

DROP TABLE IF EXISTS `user`;
CREATE TABLE `user` (
                        `id` bigint NOT NULL COMMENT '用户唯一标识',
                        `username` varchar(64) NOT NULL COMMENT '用户名',
                        `nickname` varchar(64) NOT NULL COMMENT '用户昵称',
                        `avatar` varchar(200) DEFAULT NULL COMMENT '用户头像 URL',
                        `gender` enum('m','f','o') NOT NULL COMMENT '性别',
                        `phone` varchar(20) DEFAULT NULL COMMENT '手机号',
                        `email` varchar(100) DEFAULT NULL COMMENT '电子邮箱',
                        `gmt_create` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '用户创建时间',
                        `gmt_update` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '用户信息更新时间',
                        PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='用户表';

--
-- Dumping data for table `user`
--

INSERT INTO `user` (`id`, `username`, `nickname`, `avatar`, `gender`, `phone`, `email`, `gmt_create`, `gmt_update`) VALUES (38710,'Zk','哈哈哈哈','avatar/head.png','m','17682301768','17682301768@qq.com','2025-05-06 10:18:32','2025-05-06 10:18:32');
INSERT INTO `user` (`id`, `username`, `nickname`, `avatar`, `gender`, `phone`, `email`, `gmt_create`, `gmt_update`) VALUES (38711,'Zkk','吼吼吼','avatar/head.png','f','17682301111','17682301111@qq.com','2025-05-06 10:22:42','2025-05-06 10:22:42');