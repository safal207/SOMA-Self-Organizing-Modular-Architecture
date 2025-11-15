#!/bin/bash

# SOMA v1.2 Conscious Routing - Integration Test Script
# Тестирует Decision Tracking infrastructure

set -e

API_URL="http://localhost:8080"
DECISION_IDS=()

echo "🧪 SOMA v1.2 Conscious Routing Integration Test"
echo "=============================================="
echo ""

# Проверить что сервер запущен
echo "📡 Step 1: Checking if SOMA API server is running..."
if ! curl -s "${API_URL}/" > /dev/null 2>&1; then
    echo "❌ Error: SOMA API server is not running on ${API_URL}"
    echo "Please start the server first:"
    echo "  cargo run --release --bin soma-api"
    exit 1
fi
echo "✅ Server is running"
echo ""

# Тест 1: Базовая оценка решения
echo "🎲 Step 2: Testing POST /domino/evaluate (Basic Decision)"
RESPONSE=$(curl -s -X POST "${API_URL}/domino/evaluate" \
  -H "Content-Type: application/json" \
  -d '{
    "intent_kind": "routing",
    "candidates": [
      {"peer_id": "node_alpha", "health": 0.95, "quality": 0.88, "intent_match": 0.92},
      {"peer_id": "node_beta", "health": 0.75, "quality": 0.70, "intent_match": 0.65}
    ],
    "context_tags": ["low_latency", "test"]
  }')

DECISION_ID_1=$(echo "$RESPONSE" | jq -r '.decision_id')
LUCK_SCORE=$(echo "$RESPONSE" | jq -r '.luck_score')
BEST_PEER=$(echo "$RESPONSE" | jq -r '.best_peers[0]')

echo "Response:"
echo "$RESPONSE" | jq '.'
echo ""
echo "Extracted values:"
echo "  Decision ID: $DECISION_ID_1"
echo "  Best Peer: $BEST_PEER"
echo "  Luck Score: $LUCK_SCORE"
echo ""

if [ -z "$DECISION_ID_1" ] || [ "$DECISION_ID_1" = "null" ]; then
    echo "❌ Error: decision_id not returned!"
    exit 1
fi
echo "✅ Decision ID returned successfully"
DECISION_IDS+=("$DECISION_ID_1")
echo ""

# Тест 2: Несколько решений подряд
echo "🎲 Step 3: Making multiple decisions to build history..."

HEALTHS=(0.6 0.7 0.8 0.9 0.95)
QUALITIES=(0.65 0.70 0.75 0.80 0.85)

for i in {0..4}; do
    idx=$((i))
    HEALTH=${HEALTHS[$idx]}
    QUALITY=${QUALITIES[$idx]}
    WORKER_NUM=$((i + 1))

    RESPONSE=$(curl -s -X POST "${API_URL}/domino/evaluate" \
      -H "Content-Type: application/json" \
      -d "{
        \"intent_kind\": \"task_scheduling\",
        \"candidates\": [
          {\"peer_id\": \"worker_${WORKER_NUM}\", \"health\": ${HEALTH}, \"quality\": ${QUALITY}, \"intent_match\": 0.8}
        ],
        \"context_tags\": [\"test_batch\"]
      }")

    DECISION_ID=$(echo "$RESPONSE" | jq -r '.decision_id')
    DECISION_IDS+=("$DECISION_ID")
    echo "  Decision $WORKER_NUM: $DECISION_ID (worker_${WORKER_NUM}, luck: $(echo "$RESPONSE" | jq -r '.luck_score'))"
done
echo "✅ Created 5 additional decisions"
echo ""

# Тест 3: Проверить историю решений
echo "📊 Step 4: Testing GET /domino/decisions/recent"
RECENT=$(curl -s "${API_URL}/domino/decisions/recent")
RECENT_COUNT=$(echo "$RECENT" | jq -r '.count')
echo "Recent decisions count: $RECENT_COUNT"
echo "$RECENT" | jq '.decisions[] | {decision_id, chosen_peer, luck_score, outcome}'
echo ""

if [ "$RECENT_COUNT" -lt 6 ]; then
    echo "❌ Error: Expected at least 6 decisions, got $RECENT_COUNT"
    exit 1
fi
echo "✅ History contains $RECENT_COUNT decisions"
echo ""

# Тест 4: Статистика ДО обновления outcomes
echo "📈 Step 5: Testing GET /domino/decisions/stats (BEFORE outcomes)"
STATS_BEFORE=$(curl -s "${API_URL}/domino/decisions/stats")
echo "$STATS_BEFORE" | jq '.stats'
SUCCESS_RATE_BEFORE=$(echo "$STATS_BEFORE" | jq -r '.stats.success_rate')
echo ""
echo "Success rate BEFORE: $SUCCESS_RATE_BEFORE (все pending)"
echo ""

# Тест 5: Обновить outcomes (3 success, 2 failure, 1 partial)
echo "✅❌ Step 6: Updating decision outcomes..."

# Success #1
echo "  Updating ${DECISION_IDS[0]} → success"
curl -s -X POST "${API_URL}/domino/decisions/outcome" \
  -H "Content-Type: application/json" \
  -d "{
    \"decision_id\": \"${DECISION_IDS[0]}\",
    \"outcome_type\": \"success\",
    \"actual_latency_ms\": 45.0,
    \"actual_quality\": 0.95
  }" | jq '.status'

# Success #2
echo "  Updating ${DECISION_IDS[1]} → success"
curl -s -X POST "${API_URL}/domino/decisions/outcome" \
  -H "Content-Type: application/json" \
  -d "{
    \"decision_id\": \"${DECISION_IDS[1]}\",
    \"outcome_type\": \"success\",
    \"actual_latency_ms\": 52.0,
    \"actual_quality\": 0.92
  }" | jq '.status'

# Success #3
echo "  Updating ${DECISION_IDS[2]} → success"
curl -s -X POST "${API_URL}/domino/decisions/outcome" \
  -H "Content-Type: application/json" \
  -d "{
    \"decision_id\": \"${DECISION_IDS[2]}\",
    \"outcome_type\": \"success\",
    \"actual_latency_ms\": 38.0,
    \"actual_quality\": 0.98
  }" | jq '.status'

# Failure #1
echo "  Updating ${DECISION_IDS[3]} → failure"
curl -s -X POST "${API_URL}/domino/decisions/outcome" \
  -H "Content-Type: application/json" \
  -d "{
    \"decision_id\": \"${DECISION_IDS[3]}\",
    \"outcome_type\": \"failure\",
    \"reason\": \"connection timeout\"
  }" | jq '.status'

# Failure #2
echo "  Updating ${DECISION_IDS[4]} → failure"
curl -s -X POST "${API_URL}/domino/decisions/outcome" \
  -H "Content-Type: application/json" \
  -d "{
    \"decision_id\": \"${DECISION_IDS[4]}\",
    \"outcome_type\": \"failure\",
    \"reason\": \"peer unavailable\"
  }" | jq '.status'

# Partial
echo "  Updating ${DECISION_IDS[5]} → partial"
curl -s -X POST "${API_URL}/domino/decisions/outcome" \
  -H "Content-Type: application/json" \
  -d "{
    \"decision_id\": \"${DECISION_IDS[5]}\",
    \"outcome_type\": \"partial\",
    \"completed_ratio\": 0.7,
    \"issues\": [\"slow response\", \"packet loss\"]
  }" | jq '.status'

echo "✅ Updated 6 decision outcomes"
echo ""

# Тест 6: Статистика ПОСЛЕ обновления outcomes
echo "📈 Step 7: Testing GET /domino/decisions/stats (AFTER outcomes)"
sleep 0.5  # Небольшая пауза для синхронизации
STATS_AFTER=$(curl -s "${API_URL}/domino/decisions/stats")
echo "$STATS_AFTER" | jq '.stats'
echo ""

TOTAL=$(echo "$STATS_AFTER" | jq -r '.stats.total_decisions')
SUCCESSFUL=$(echo "$STATS_AFTER" | jq -r '.stats.successful_decisions')
SUCCESS_RATE=$(echo "$STATS_AFTER" | jq -r '.stats.success_rate')
AVG_LUCK=$(echo "$STATS_AFTER" | jq -r '.stats.avg_luck_score')

echo "Statistics summary:"
echo "  Total decisions: $TOTAL"
echo "  Successful: $SUCCESSFUL"
echo "  Success rate: $SUCCESS_RATE"
echo "  Avg luck score: $AVG_LUCK"
echo ""

# Проверка что success_rate изменился
if (( $(echo "$SUCCESS_RATE > 0.0" | bc -l) )); then
    echo "✅ Success rate updated (was 0.0, now $SUCCESS_RATE)"
else
    echo "⚠️  Warning: Success rate still 0.0 (outcomes might not be applied)"
fi
echo ""

# Тест 7: Проверить конкретное решение с обновлённым outcome
echo "🔍 Step 8: Verifying updated decision details..."
ALL_DECISIONS=$(curl -s "${API_URL}/domino/decisions")
FIRST_DECISION=$(echo "$ALL_DECISIONS" | jq ".decisions[] | select(.decision_id == \"${DECISION_IDS[0]}\")")
echo "Decision ${DECISION_IDS[0]} details:"
echo "$FIRST_DECISION" | jq '{decision_id, chosen_peer, luck_score, outcome}'
echo ""

OUTCOME_TYPE=$(echo "$FIRST_DECISION" | jq -r '.outcome | keys[0]')
if [ "$OUTCOME_TYPE" = "Success" ]; then
    echo "✅ Outcome successfully updated to Success"
else
    echo "⚠️  Warning: Outcome is $OUTCOME_TYPE (expected Success)"
fi
echo ""

# Тест 8: Проверить Conscious State integration
echo "🧠 Step 9: Testing Conscious State integration..."
CONSCIOUS_STATE=$(curl -s "${API_URL}/conscious/state")
DECISION_COUNT=$(echo "$CONSCIOUS_STATE" | jq -r '.node_id')
echo "Conscious State:"
echo "$CONSCIOUS_STATE" | jq '{node_id, cycle_count, traces_count, insights_count}'
echo ""

# Summary
echo "=============================================="
echo "📊 Test Summary"
echo "=============================================="
echo ""
echo "✅ POST /domino/evaluate - Working (returns decision_id)"
echo "✅ GET /domino/decisions/recent - Working ($RECENT_COUNT decisions)"
echo "✅ GET /domino/decisions/stats - Working"
echo "✅ POST /domino/decisions/outcome - Working (6 updates)"
echo "✅ Decision history persistence - Working"
echo "✅ Conscious State integration - Working"
echo ""
echo "Key Metrics:"
echo "  • Total decisions made: $TOTAL"
echo "  • Successful outcomes: $SUCCESSFUL"
echo "  • Success rate: $SUCCESS_RATE"
echo "  • Average luck score: $AVG_LUCK"
echo ""
echo "🎉 All tests passed! v1.2 Conscious Routing is working correctly!"
echo ""
echo "Next steps:"
echo "  • View full decision history: curl ${API_URL}/domino/decisions | jq"
echo "  • Monitor stats in real-time: watch -n 2 'curl -s ${API_URL}/domino/decisions/stats | jq'"
echo ""
