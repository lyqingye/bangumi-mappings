// API响应状态码
export const API_STATUS = {
  SUCCESS: 200,
  CREATED: 201,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  SERVER_ERROR: 500,
}

// 请求配置
export const REQUEST_CONFIG = {
  TIMEOUT: 10000, // 10秒
  RETRY_COUNT: 3, // 重试次数
  RETRY_DELAY: 1000, // 重试延迟（毫秒）
}

