from flask import Blueprint, jsonify, request
import violence_governance_core as vgc
import os
import json
import uuid
import datetime
from baidu_nlp import BaiduNLP

# 初始化蓝图
api_bp = Blueprint('api', __name__, url_prefix='/api/v1')

# 初始化文本分析器
text_analyzer = vgc.TextAnalyzer()

# 初始化百度NLP客户端
api_key = os.environ.get('BAIDU_API_KEY', 'your_default_api_key')
secret_key = os.environ.get('BAIDU_SECRET_KEY', 'your_default_secret_key')
baidu_nlp = BaiduNLP(api_key, secret_key)

# 简单的话题存储（实际应用中会使用数据库）
topics = {}


@api_bp.route('/analyze', methods=['POST'])
def analyze_content():
    """
    分析内容API端点
    ---
    请求:
      content: 待分析的文本内容
      content_type: 内容类型（目前仅支持"text"）
    响应:
      micro_analysis: 微观内容分析结果
      micro_action: 建议的干预措施
      macro_analysis: 宏观话题分析
      macro_interventions: 话题级干预建议
    """
    data = request.json

    if not data or 'content' not in data:
        return jsonify({
            'error': 'Missing content field',
            'status': 'error',
            'code': 400
        }), 400

    content = data['content']
    content_type = data.get('content_type', 'text')

    if content_type != 'text':
        return jsonify({
            'error': f'Unsupported content type: {content_type}',
            'status': 'error',
            'code': 400
        }), 400

    try:
        # 尝试调用百度API进行分析
        sentiment_result = None
        keywords = []

        try:
            # 情感分析
            sentiment_result = baidu_nlp.sentiment_analyze(content)

            # 关键词提取
            try:
                keyword_items = baidu_nlp.keyword_extract(content, 5)
                keywords = [item["word"] for item in keyword_items]
            except Exception as e:
                # 使用简单分词作为回退
                keywords = [word for word in content.lower().split() if len(word) > 3]

        except Exception as e:
            # 使用本地简单计算作为回退
            local_sentiment = {
                "sentiment": 1,  # 默认中性
                "confidence": 0.5,
                "positive_prob": 0.5,
                "negative_prob": 0.5
            }

            # 简单负面检测
            if any(word in content.lower() for word in ["讨厌", "恨", "烦", "滚", "垃圾"]):
                local_sentiment["sentiment"] = 0  # 消极
                local_sentiment["positive_prob"] = 0.2
                local_sentiment["negative_prob"] = 0.8

            sentiment_result = local_sentiment

        # 微观内容分析 - 确保参数类型正确
        api_key = "demo_key"  # 简化处理

        # 计算本地暴力分数
        local_violence_score = 0.0
        if "讨厌" in content.lower(): local_violence_score += 0.3
        if "恨" in content.lower(): local_violence_score += 0.4
        if "威胁" in content.lower(): local_violence_score += 0.5
        if "滚" in content.lower(): local_violence_score += 0.4
        if "垃圾" in content.lower(): local_violence_score += 0.3
        local_violence_score = min(local_violence_score, 1.0)

        # 确定暴力类型
        violence_type = None
        if local_violence_score > 0.7:
            violence_type = "harassment"

        # 使用简化版的analyze_text调用，不传递复杂参数
        analysis = text_analyzer.analyze_text(content, api_key)

        # 手动设置百度API分析结果
        if sentiment_result:
            if "sentiment" in sentiment_result:
                analysis.sentiment = sentiment_result["sentiment"]
            if "positive_prob" in sentiment_result:
                analysis.positive_prob = sentiment_result["positive_prob"]
            if "negative_prob" in sentiment_result:
                analysis.negative_prob = sentiment_result["negative_prob"]

        # 手动设置关键词
        analysis.keywords = keywords

        # 调整暴力分数，结合百度API结果
        if sentiment_result and "negative_prob" in sentiment_result:
            analysis.violence_score = (local_violence_score + sentiment_result["negative_prob"]) / 2
        else:
            analysis.violence_score = local_violence_score

        if violence_type:
            analysis.violence_type = violence_type

        # 应用规则引擎
        action = vgc.determine_action(analysis)

        # 生成话题ID
        topic_id = f"topic-{str(uuid.uuid4())[:8]}"

        # 计算风险评分
        violence_risk_score = analysis.violence_score
        if hasattr(analysis, 'negative_prob') and analysis.negative_prob is not None:
            # 融合消极概率到风险评分
            violence_risk_score = (violence_risk_score + analysis.negative_prob) / 2

        # 基于规则引擎的严重程度调整风险评分
        if action.severity == "critical":
            violence_risk_score = max(violence_risk_score, 0.9)
        elif action.severity == "high":
            violence_risk_score = max(violence_risk_score, 0.7)
        elif action.severity == "medium":
            violence_risk_score = max(violence_risk_score, 0.5)

        # 设置干预状态
        intervention_status = "Monitoring"
        if action.severity == "critical":
            intervention_status = "ActiveIntervention"
        elif action.severity == "high":
            intervention_status = "EarlyWarning"
        elif action.severity == "medium" and violence_risk_score > 0.5:
            intervention_status = "EarlyWarning"

        # 创建话题分析结果
        topic_analysis = {
            "topic_id": topic_id,
            "keywords": keywords,
            "start_time": datetime.datetime.now(datetime.UTC).isoformat(),
            "last_update": datetime.datetime.now(datetime.UTC).isoformat(),
            "sentiment_score": -analysis.violence_score,
            "negativity_ratio": getattr(analysis, 'negative_prob', analysis.violence_score),
            "violence_risk_score": violence_risk_score,
            "content_count": 1,
            "users_involved": 1,
            "growth_rate": 0.0,
            "intervention_status": intervention_status
        }

        # 存储话题
        topics[topic_id] = topic_analysis

        # 生成干预建议
        macro_interventions = []

        # 基于严重程度生成干预建议
        if action.severity == "critical":
            macro_interventions = [
                {
                    "strategy_type": "ContentFiltering",
                    "priority": 9,
                    "target_audience": "AllUsers",
                    "description": "紧急：为该话题启用最严格内容过滤，阻止相关内容传播",
                    "estimated_impact": 0.9
                },
                {
                    "strategy_type": "ModeratorAssignment",
                    "priority": 8,
                    "target_audience": "AllUsers",
                    "description": "立即分配专职版主监控该话题，进行人工审核",
                    "estimated_impact": 0.8
                }
            ]
        elif action.severity == "high":
            macro_interventions = [
                {
                    "strategy_type": "ContentFiltering",
                    "priority": 7,
                    "target_audience": "AllUsers",
                    "description": "为该话题启用严格内容过滤，限制可见性",
                    "estimated_impact": 0.7
                },
                {
                    "strategy_type": "UserWarning",
                    "priority": 6,
                    "target_audience": "ActiveParticipants",
                    "description": "向参与讨论的用户发送提醒，避免情绪激化",
                    "estimated_impact": 0.6
                }
            ]
        elif action.severity == "medium" or violence_risk_score > 0.5:
            macro_interventions = [
                {
                    "strategy_type": "PositiveContentPromotion",
                    "priority": 5,
                    "target_audience": "AllUsers",
                    "description": "在该话题中推广积极内容，平衡讨论氛围",
                    "estimated_impact": 0.5
                },
                {
                    "strategy_type": "AlgorithmAdjustment",
                    "priority": 4,
                    "target_audience": "AllUsers",
                    "description": "适当调整内容推荐算法，减少负面内容曝光",
                    "estimated_impact": 0.4
                }
            ]
        elif violence_risk_score > 0.3:
            macro_interventions = [
                {
                    "strategy_type": "Monitoring",
                    "priority": 3,
                    "target_audience": "SystemOnly",
                    "description": "加强对该话题的监控，定期评估风险变化",
                    "estimated_impact": 0.3
                }
            ]

        # 构建返回结果
        result = {
            'status': 'success',
            'code': 200,
            'data': {
                'micro_analysis': {
                    'content_id': analysis.content_id,
                    'content_type': analysis.content_type,
                    'violence_score': analysis.violence_score,
                    'violence_type': analysis.violence_type,
                    'confidence_score': analysis.confidence_score,
                    'is_violent': analysis.is_violent(),
                    'is_negative': analysis.is_negative(),
                    'sentiment': getattr(analysis, 'sentiment', None),
                    'positive_prob': getattr(analysis, 'positive_prob', None),
                    'negative_prob': getattr(analysis, 'negative_prob', None),
                    'keywords': getattr(analysis, 'keywords', [])
                },
                'micro_action': {
                    'action_type': action.action_type,
                    'severity': action.severity,
                    'message': action.message,
                    'automated': action.automated
                },
                'macro_analysis': topic_analysis,
                'macro_interventions': macro_interventions
            }
        }

        return jsonify(result)

    except Exception as e:
        # 统一的错误处理
        return jsonify({
            'error': str(e),
            'status': 'error',
            'code': 500
        }), 500


@api_bp.route('/topics/<topic_id>', methods=['GET'])
def get_topic(topic_id):
    """
    获取话题分析结果API端点
    ---
    请求参数:
      topic_id: 话题ID
    响应:
      话题分析结果
    """
    if topic_id not in topics:
        return jsonify({
            'error': f'Topic not found: {topic_id}',
            'status': 'error',
            'code': 404
        }), 404

    return jsonify({
        'status': 'success',
        'code': 200,
        'data': topics[topic_id]
    })


@api_bp.route('/topics', methods=['GET'])
def list_topics():
    """
    获取所有话题列表API端点
    ---
    响应:
      话题列表
    """
    return jsonify({
        'status': 'success',
        'code': 200,
        'data': {
            'topics': list(topics.values()),
            'count': len(topics)
        }
    })


@api_bp.route('/status', methods=['GET'])
def api_status():
    """
    API状态检查端点
    ---
    响应:
      服务状态信息
    """
    return jsonify({
        'status': 'success',
        'code': 200,
        'data': {
            'service': 'network_violence_governance_api',
            'version': '1.0.0',
            'status': 'running',
            'timestamp': datetime.datetime.now(datetime.UTC).isoformat()
        }
    })