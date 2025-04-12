import { QueryClient } from "@tanstack/react-query"

// 创建一个 QueryClient 实例
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // 默认缓存时间设置为 5 分钟
      staleTime: 5 * 60 * 1000,
      // 默认缓存保留时间设置为 60 分钟
      gcTime: 60 * 60 * 1000,
      // 默认重试次数
      retry: 1,
      // 默认启用重新获取
      refetchOnWindowFocus: true,
    },
  },
})

