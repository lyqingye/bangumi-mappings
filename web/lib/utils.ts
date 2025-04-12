import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { ReviewStatus } from "./types"

/**
 * 合并Tailwind类名
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 获取验证状态的标签和颜色
 */
export function getStatusLabel(status: ReviewStatus) {
  switch (status) {
    case ReviewStatus.Accepted:
      return { label: "已接受", color: "bg-green-500" }
    case ReviewStatus.Rejected:
      return { label: "已拒绝", color: "bg-red-500" }
    case ReviewStatus.Dropped:
      return { label: "已丢弃", color: "bg-yellow-500" }
    case ReviewStatus.Ready:
      return { label: "待验证", color: "bg-blue-500" }
    case ReviewStatus.UnMatched:
      return { label: "未匹配", color: "bg-gray-500" }
  }
}

/**
 * 格式化日期
 */
export function formatDate(dateString: string): string {
  if (!dateString) return "未知日期"

  try {
    const date = new Date(dateString)
    return date.toLocaleDateString("zh-CN", {
      year: "numeric",
      month: "long",
      day: "numeric",
    })
  } catch (error) {
    return dateString
  }
}

