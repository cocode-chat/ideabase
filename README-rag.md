
## 召回测试

```shell
curl --location --request POST 'http://localhost:8080/api/v1/rag/recall.json' \
--header 'Content-Type: application/json;chatset=UTF8' \
--data-raw '{
    "collection": "ecommerce",
    "message": "钻石耳钉套装"
}'
```
```json
{
    "code": 200,
    "data": [
        {
            "page_content": "\"category: 钻石首饰\\nid: 3\\nname: 钻石耳钉套装\\nprice: 2999.00\\nstock_quantity: 35\\ndescription: 18K黄金镶嵌0.3克拉钻石耳钉，精致小巧\\n\"",
            "metadata": {
                "category": "钻石首饰",
                "item_id": 3,
                "src_type": "item"
            },
            "score": 0.8167279362678528
        },
        {
            "page_content": "\"total_amount: 16899.00\\nid: 6\\ncustomer_id: 1006\\norder_date: 2024-10-06 18:30:05\\nstatus: processing\\nshipping_address: 杭州市西湖区文三路90号\\n商品列表: \\n - item_name: 钻石耳钉套装 price_per_item: 2999.00 id: 6 quantity: 1 \\n - id: 6 quantity: 1 price_per_item: 18999.00 item_name: 豪华钻石戒指 \"",
            "metadata": {
                "order_id": 6,
                "src_type": "order"
            },
            "score": 0.7920307517051697
        },
        {
            "page_content": "\"status: processing\\nid: 2\\norder_date: 2024-10-02 14:22:10\\ntotal_amount: 6498.00\\nshipping_address: 上海市浦东新区陆家嘴环路1000号\\ncustomer_id: 1002\\n商品列表: \\n - item_name: 0.5克拉钻石戒指 id: 2 quantity: 1 price_per_item: 4599.00 \\n - item_name: 钻石耳钉套装 quantity: 1 price_per_item: 2999.00 id: 2 \"",
            "metadata": {
                "order_id": 2,
                "src_type": "order"
            },
            "score": 0.7120480537414551
        }
    ]
}
```

## 检索对话

```shell
curl --location --request POST 'http://localhost:8080/api/v1/rag/conversation.json' \
--header 'Content-Type: application/json' \
--data-raw '{
    "collection": "ecommerce",
    "message": "1001用户购买了多少单，总共花了多少钱？最近的订单状态是什么？"
}'
```
```json
{
  "code": 200,
  "data": "根据提供的订单信息，用户 **1001** 的相关信息如下：\n\n1. 用户 1001 只有一笔订单记录（`id: 1`）。\n2. 这笔订单的总金额为 **13898.00** 元。\n3. 最近的订单状态为 **shipped**（已发货）。\n\n因此，答案是：\n- 用户 1001 购买了 **1单**。\n- 总共花费了 **13898.00元**。\n- 最近的订单状态是 **shipped**。"
}
```