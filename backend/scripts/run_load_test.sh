#!/bin/bash

# Configuration
K6_IMAGE="grafana/k6:latest"
TEST_DIR="tests/load"
RESULTS_DIR="test-results"
DATE=$(date +%Y%m%d_%H%M%S)

# Create results directory
mkdir -p $RESULTS_DIR

# Function to run a load test scenario
run_load_test() {
    local scenario=$1
    local duration=$2
    local vus=$3
    local output_file="$RESULTS_DIR/${scenario}_${DATE}.json"
    local html_report="$RESULTS_DIR/${scenario}_${DATE}.html"

    echo "Running $scenario load test..."
    echo "Duration: $duration"
    echo "Virtual Users: $vus"

    # Run k6 test
    docker run --rm \
        -v $(pwd)/$TEST_DIR:/scripts \
        -v $(pwd)/$RESULTS_DIR:/results \
        $K6_IMAGE run \
        --out json=/results/$(basename $output_file) \
        --out html=/results/$(basename $html_report) \
        -e DURATION=$duration \
        -e VUS=$vus \
        /scripts/$scenario.js

    echo "Test completed. Results saved to $output_file and $html_report"
}

# Create load test scenarios directory
mkdir -p $TEST_DIR

# Create basic load test scenario
cat > $TEST_DIR/basic.js << 'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    vus: __ENV.VUS || 10,
    duration: __ENV.DURATION || '30s',
};

export default function () {
    // Test health endpoint
    const healthRes = http.get('http://localhost:3000/health');
    check(healthRes, {
        'health status is 200': (r) => r.status === 200,
    });

    // Test message sending
    const messagePayload = JSON.stringify({
        content: 'Test message',
        sender_id: 1,
        receiver_id: 2,
    });

    const sendRes = http.post('http://localhost:3000/messages', messagePayload, {
        headers: { 'Content-Type': 'application/json' },
    });

    check(sendRes, {
        'send message status is 201': (r) => r.status === 201,
    });

    sleep(1);
}
EOF

# Create stress test scenario
cat > $TEST_DIR/stress.js << 'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
    stages: [
        { duration: '1m', target: 50 },  // Ramp up to 50 users
        { duration: '3m', target: 50 },  // Stay at 50 users
        { duration: '1m', target: 100 }, // Ramp up to 100 users
        { duration: '3m', target: 100 }, // Stay at 100 users
        { duration: '1m', target: 0 },   // Ramp down to 0 users
    ],
    thresholds: {
        'errors': ['rate<0.1'], // Error rate should be less than 10%
        'http_req_duration': ['p(95)<500'], // 95% of requests should be below 500ms
    },
};

export default function () {
    const messagePayload = JSON.stringify({
        content: 'Stress test message',
        sender_id: Math.floor(Math.random() * 100) + 1,
        receiver_id: Math.floor(Math.random() * 100) + 1,
    });

    const responses = http.batch([
        ['GET', 'http://localhost:3000/health'],
        ['POST', 'http://localhost:3000/messages', messagePayload, {
            headers: { 'Content-Type': 'application/json' },
        }],
    ]);

    responses.forEach((res) => {
        errorRate.add(res.status !== 200 && res.status !== 201);
    });

    sleep(1);
}
EOF

# Create spike test scenario
cat > $TEST_DIR/spike.js << 'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    scenarios: {
        spike: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '10s', target: 100 }, // Spike to 100 users
                { duration: '1m', target: 100 },  // Stay at 100 users
                { duration: '10s', target: 0 },   // Ramp down to 0 users
            ],
        },
    },
};

export default function () {
    const messagePayload = JSON.stringify({
        content: 'Spike test message',
        sender_id: Math.floor(Math.random() * 100) + 1,
        receiver_id: Math.floor(Math.random() * 100) + 1,
    });

    const responses = http.batch([
        ['GET', 'http://localhost:3000/health'],
        ['POST', 'http://localhost:3000/messages', messagePayload, {
            headers: { 'Content-Type': 'application/json' },
        }],
    ]);

    responses.forEach((res) => {
        check(res, {
            'status is 200 or 201': (r) => r.status === 200 || r.status === 201,
        });
    });

    sleep(1);
}
EOF

# Run load tests
echo "Starting load tests..."

# Basic load test
run_load_test "basic" "1m" "10"

# Stress test
run_load_test "stress" "8m" "100"

# Spike test
run_load_test "spike" "1m20s" "100"

echo "All load tests completed. Results are available in $RESULTS_DIR" 