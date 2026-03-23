# Vibe Kanban 内网部署指南

本文档介绍如何使用 Docker 将 Vibe Kanban 部署到内网环境。

## 环境要求

- Docker 20.10+
- Docker Compose 2.0+ (可选，使用 docker-compose 时)

## 快速部署

### 1. 构建镜像

```bash
# 在项目根目录下执行
docker build -t vibe-kanban:latest .
```

### 2. 使用 Docker Compose 运行

```bash
# 复制环境变量文件
cp .env.docker .env

# 启动服务
docker compose -f docker-compose.local.yml up -d

# 查看日志
docker compose -f docker-compose.local.yml logs -f
```

### 3. 访问应用

打开浏览器访问: `http://localhost:3000`

## 配置说明

### 环境变量 (.env)

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `PORT` | 服务端口 | `3000` |
| `RUST_LOG` | 日志级别 (info, debug, warn, error) | `info` |
| `POSTHOG_API_KEY` | PostHog 分析 API Key (留空则禁用) | 空 |
| `POSTHOG_API_ENDPOINT` | PostHog 服务地址 | 空 |

### 端口修改

修改 `.env` 文件中的 `PORT`：

```env
PORT=8080
```

同时修改 `docker-compose.local.yml` 中的端口映射：

```yaml
ports:
  - "8080:3000"
```

## 数据持久化

数据存储在 Docker 卷 `vibe-data` 中，包含：
- SQLite 数据库
- 配置文件

### 查看数据卷位置

```bash
docker volume inspect vibe-kanban_vibe-data
```

### 备份数据

```bash
# 导出数据卷
docker run --rm -v vibe-kanban_vibe-data:/data -v $(pwd):/backup alpine tar czf /backup/vibe-data-backup.tar.gz -C /data .

# 恢复数据
docker run --rm -v vibe-kanban_vibe-data:/data -v $(pwd):/backup alpine tar xzf /backup/vibe-data-backup.tar.gz -C /data
```

## 导出镜像用于内网服务器

### 1. 保存镜像

```bash
# 保存镜像到 tar 文件
docker save vibe-kanban:latest -o vibe-kanban.tar
```

### 2. 传输到内网服务器

使用 U 盘、移动硬盘或其他方式将 `vibe-kanban.tar` 传输到内网服务器。

### 3. 在内网服务器加载镜像

```bash
docker load -i vibe-kanban.tar
```

### 4. 在内网服务器运行

```bash
# 创建工作目录
mkdir -p ~/vibe-kanban
cd ~/vibe-kanban

# 创建 docker-compose.yml
vim docker-compose.yml
```

`docker-compose.yml` 内容：

```yaml
services:
  vibe-kanban:
    image: vibe-kanban:latest
    ports:
      - "3000:3000"
    environment:
      RUST_LOG: info
      HOST: 0.0.0.0
      PORT: 3000
    volumes:
      - vibe-data:/root/.local/share/ai.bloop.vibe-kanban
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

volumes:
  vibe-data:
```

### 5. 启动服务

```bash
# 启动
docker compose up -d

# 查看日志
docker compose logs -f

# 停止服务
docker compose down

# 重启服务
docker compose restart
```

## 常用命令

```bash
# 启动服务
docker compose -f docker-compose.local.yml up -d

# 停止服务
docker compose -f docker-compose.local.yml down

# 重启服务
docker compose -f docker-compose.local.yml restart

# 查看日志
docker compose -f docker-compose.local.yml logs -f

# 查看服务状态
docker compose -f docker-compose.local.yml ps

# 进入容器
docker compose -f docker-compose.local.yml exec vibe-kanban sh

# 重建并启动
docker compose -f docker-compose.local.yml up -d --build
```

## 故障排查

### 服务无法启动

```bash
# 查看详细日志
docker compose -f docker-compose.local.yml logs

# 检查端口是否被占用
netstat -tlnp | grep 3000
```

### 数据库问题

```bash
# 进入容器检查
docker compose -f docker-compose.local.yml exec vibe-kanban sh

# 查看数据库文件
ls -la /root/.local/share/ai.bloop.vibe-kanban/

# 重建数据卷 (会丢失数据)
docker compose -f docker-compose.local.yml down -v
docker compose -f docker-compose.local.yml up -d
```

### 网络问题

```bash
# 检查防火墙规则
firewall-cmd --list-all  # CentOS/RHEL
ufw status               # Ubuntu
```

## 注意事项

1. **内网访问**: 默认配置仅允许本机访问。如需局域网访问，将端口映射改为 `0.0.0.0:3000:3000`

2. **数据安全**: 建议定期备份数据卷，防止数据丢失

3. **性能**: 对于多用户使用场景，建议增加 Docker 资源限制

4. **更新**: 重新构建镜像后，使用 `docker compose up -d --build` 重新部署
