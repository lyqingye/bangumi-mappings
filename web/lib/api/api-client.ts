/**
 * API客户端 - 集中管理所有API请求
 */

// API基础URL - 从环境变量获取
const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080/api"

// 请求超时时间（毫秒）
const REQUEST_TIMEOUT = 10000

// API响应格式
interface ApiResponse<T> {
  code: number
  msg: string
  data: T
}

/**
 * 带超时的fetch函数
 */
async function fetchWithTimeout(url: string, options: RequestInit = {}, timeout = REQUEST_TIMEOUT): Promise<Response> {
  const controller = new AbortController()
  const { signal } = controller

  // 创建一个超时Promise
  const timeoutPromise = new Promise<never>((_, reject) => {
    const timeoutId = setTimeout(() => {
      controller.abort()
      reject(new Error(`Request timeout after ${timeout}ms`))
    }, timeout)

    // 清除超时计时器
    if (signal) {
      signal.addEventListener("abort", () => clearTimeout(timeoutId))
    }
  })

  // 创建fetch Promise
  const fetchPromise = fetch(url, {
    ...options,
    signal,
  })

  // 竞争两个Promise
  return Promise.race([fetchPromise, timeoutPromise])
}

/**
 * 处理API响应
 */
async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const errorText = await response.text()
    console.error(`API error (${response.status}):`, errorText)
    throw new Error(`API error: ${response.status} ${response.statusText}`)
  }

  const jsonResponse = (await response.json()) as ApiResponse<T>

  // 检查API响应状态码
  if (jsonResponse.code !== 0) {
    console.error(`API business error (${jsonResponse.code}):`, jsonResponse.msg)
    throw new Error(`API business error: ${jsonResponse.msg}`)
  }

  // 返回data字段
  return jsonResponse.data
}

/**
 * API客户端
 */
export const apiClient = {
  /**
   * 发送GET请求
   */
  async get<T>(endpoint: string): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`

    try {
      const response = await fetchWithTimeout(url, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      })

      return handleResponse<T>(response)
    } catch (error) {
      console.error(`Error fetching ${url}:`, error)
      throw error
    }
  },

  /**
   * 发送POST请求
   */
  async post<T>(endpoint: string, data: any): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`

    try {
      const response = await fetchWithTimeout(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
      })

      return handleResponse<T>(response)
    } catch (error) {
      console.error(`Error posting to ${url}:`, error)
      throw error
    }
  },

  /**
   * 发送PUT请求
   */
  async put<T>(endpoint: string, data: any): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`

    try {
      const response = await fetchWithTimeout(url, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
      })

      return handleResponse<T>(response)
    } catch (error) {
      console.error(`Error putting to ${url}:`, error)
      throw error
    }
  },

  /**
   * 发送DELETE请求
   */
  async delete<T>(endpoint: string): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`

    try {
      const response = await fetchWithTimeout(url, {
        method: "DELETE",
        headers: {
          "Content-Type": "application/json",
        },
      })

      return handleResponse<T>(response)
    } catch (error) {
      console.error(`Error deleting ${url}:`, error)
      throw error
    }
  },
}

