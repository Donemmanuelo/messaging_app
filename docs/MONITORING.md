# Monitoring Guide

## Overview

The application uses Prometheus for metrics collection and Grafana for visualization. This guide covers:
- Key metrics to monitor
- Alert configurations
- Dashboard setup
- Performance monitoring
- Security monitoring

## Metrics

### Application Metrics

#### Request Metrics
```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m])

# Response time
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

#### WebSocket Metrics
```promql
# Active connections
websocket_connections_total

# Message rate
rate(websocket_messages_total[5m])

# Connection errors
rate(websocket_errors_total[5m])
```

#### Database Metrics
```promql
# Query rate
rate(pg_stat_statements_calls[5m])

# Transaction rate
rate(pg_stat_database_xact_commit[5m])

# Cache hit ratio
pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read)
```

#### Redis Metrics
```promql
# Command rate
rate(redis_commands_processed_total[5m])

# Memory usage
redis_memory_used_bytes

# Connected clients
redis_connected_clients
```

### System Metrics

#### CPU Usage
```promql
# CPU utilization
rate(process_cpu_seconds_total[5m])

# Load average
node_load1
```

#### Memory Usage
```promql
# Memory utilization
process_resident_memory_bytes

# Heap usage
process_heap_bytes
```

#### Disk Usage
```promql
# Disk space
node_filesystem_size_bytes - node_filesystem_free_bytes

# IO rate
rate(node_disk_io_time_seconds_total[5m])
```

#### Network Usage
```promql
# Network traffic
rate(node_network_transmit_bytes_total[5m])
rate(node_network_receive_bytes_total[5m])

# Network errors
rate(node_network_transmit_errs_total[5m])
rate(node_network_receive_errs_total[5m])
```

## Alerts

### Critical Alerts

```yaml
groups:
  - name: critical
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) * 100 > 5
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate
          description: Error rate is above 5% for 5 minutes

      - alert: DatabaseDown
        expr: pg_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: Database is down
          description: PostgreSQL is not responding

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes / process_virtual_memory_bytes * 100 > 90
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High memory usage
          description: Memory usage is above 90%
```

### Warning Alerts

```yaml
groups:
  - name: warning
    rules:
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High request latency
          description: 95th percentile of request latency is above 1 second

      - alert: HighCPUUsage
        expr: rate(process_cpu_seconds_total[5m]) * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High CPU usage
          description: CPU usage is above 80%

      - alert: DiskSpaceLow
        expr: (node_filesystem_size_bytes - node_filesystem_free_bytes) / node_filesystem_size_bytes * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Low disk space
          description: Disk space usage is above 85%
```

## Dashboards

### Application Overview

```json
{
  "dashboard": {
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{path}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
            "legendFormat": "{{status}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p95"
          }
        ]
      }
    ]
  }
}
```

### Database Performance

```json
{
  "dashboard": {
    "panels": [
      {
        "title": "Query Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(pg_stat_statements_calls[5m])",
            "legendFormat": "{{query}}"
          }
        ]
      },
      {
        "title": "Cache Hit Ratio",
        "type": "gauge",
        "targets": [
          {
            "expr": "pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read) * 100",
            "legendFormat": "Hit Ratio"
          }
        ]
      },
      {
        "title": "Transaction Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(pg_stat_database_xact_commit[5m])",
            "legendFormat": "Commits"
          }
        ]
      }
    ]
  }
}
```

### System Health

```json
{
  "dashboard": {
    "panels": [
      {
        "title": "CPU Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(process_cpu_seconds_total[5m]) * 100",
            "legendFormat": "CPU %"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "process_resident_memory_bytes",
            "legendFormat": "Memory"
          }
        ]
      },
      {
        "title": "Disk Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "(node_filesystem_size_bytes - node_filesystem_free_bytes) / node_filesystem_size_bytes * 100",
            "legendFormat": "Usage %"
          }
        ]
      }
    ]
  }
}
```

## Performance Monitoring

### Load Testing

```bash
# Run load test
./scripts/run_load_test.sh

# Monitor during test
watch -n 1 'curl -s http://localhost:9090/api/v1/query?query=rate\(http_requests_total\[5m\]\)'
```

### Profiling

```bash
# CPU profile
cargo flamegraph

# Memory profile
cargo heaptrack
```

### Tracing

```rust
#[instrument]
pub async fn handle_request(req: Request) -> Result<Response, Error> {
    // Request handling logic
}
```

## Security Monitoring

### Security Metrics

```promql
# Failed login attempts
rate(auth_failed_attempts_total[5m])

# Rate limit hits
rate(rate_limit_hits_total[5m])

# Suspicious IPs
rate(suspicious_ip_blocks_total[5m])
```

### Security Alerts

```yaml
groups:
  - name: security
    rules:
      - alert: BruteForceAttempt
        expr: rate(auth_failed_attempts_total[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: Brute force attempt detected
          description: More than 10 failed login attempts per minute

      - alert: SuspiciousActivity
        expr: rate(suspicious_ip_blocks_total[5m]) > 0
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: Suspicious activity detected
          description: IP addresses have been blocked due to suspicious activity
```

## Logging

### Log Configuration

```yaml
logging:
  level: info
  format: json
  output: stdout
  fields:
    service: messaging-app
    environment: production
```

### Log Analysis

```bash
# View application logs
docker-compose logs -f backend

# Search for errors
docker-compose logs backend | grep ERROR

# Analyze log patterns
docker-compose logs backend | jq 'select(.level == "ERROR")'
```

## Maintenance

### Regular Tasks

1. **Daily**:
   - Review critical alerts
   - Check error rates
   - Monitor resource usage

2. **Weekly**:
   - Review performance metrics
   - Analyze error patterns
   - Check security alerts

3. **Monthly**:
   - Review dashboard effectiveness
   - Update alert thresholds
   - Optimize monitoring setup

### Optimization

1. **Metrics**:
   - Remove unused metrics
   - Optimize query performance
   - Adjust scrape intervals

2. **Alerts**:
   - Fine-tune thresholds
   - Reduce alert noise
   - Improve alert messages

3. **Dashboards**:
   - Update visualizations
   - Add new metrics
   - Optimize queries
``` 