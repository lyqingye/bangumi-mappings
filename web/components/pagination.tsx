"use client"

import { ChevronLeft, ChevronRight } from "lucide-react"
import { Button } from "@/components/ui/button"

interface PaginationProps {
  currentPage: number
  totalItems: number
  pageSize: number
  onPageChange: (page: number) => void
  disabled?: boolean
}

export function Pagination({ currentPage, totalItems, pageSize, onPageChange, disabled = false }: PaginationProps) {
  // 计算总页数
  const totalPages = Math.ceil(totalItems / pageSize)

  // 生成页码数组
  const getPageNumbers = () => {
    const pages = []
    const maxPagesToShow = 5

    if (totalPages <= maxPagesToShow) {
      // 如果总页数小于等于最大显示页数，显示所有页码
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i)
      }
    } else {
      // 否则，显示当前页附近的页码
      let startPage = Math.max(1, currentPage - Math.floor(maxPagesToShow / 2))
      let endPage = startPage + maxPagesToShow - 1

      if (endPage > totalPages) {
        endPage = totalPages
        startPage = Math.max(1, endPage - maxPagesToShow + 1)
      }

      for (let i = startPage; i <= endPage; i++) {
        pages.push(i)
      }

      // 添加省略号
      if (startPage > 1) {
        pages.unshift(-1) // -1 表示省略号
        pages.unshift(1) // 始终显示第一页
      }

      if (endPage < totalPages) {
        pages.push(-2) // -2 表示省略号
        pages.push(totalPages) // 始终显示最后一页
      }
    }

    return pages
  }

  const pageNumbers = getPageNumbers()

  return (
    <div className="flex flex-col items-center justify-center mt-4">
      <div className="flex items-center justify-center space-x-2 mb-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage === 1 || disabled}
          className="bg-[#222] border-[#333] text-white"
        >
          <ChevronLeft className="h-4 w-4" />
        </Button>

        {pageNumbers.map((page, index) => {
          if (page < 0) {
            // 渲染省略号
            return (
              <span key={`ellipsis-${index}`} className="px-2 text-[#777]">
                ...
              </span>
            )
          }

          return (
            <Button
              key={page}
              variant={currentPage === page ? "default" : "outline"}
              size="sm"
              onClick={() => onPageChange(page)}
              disabled={disabled}
              className={
                currentPage === page
                  ? "bg-gradient-to-r from-purple-600 to-blue-600 text-white border-0"
                  : "bg-[#222] border-[#333] text-white"
              }
            >
              {page}
            </Button>
          )
        })}

        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage === totalPages || disabled}
          className="bg-[#222] border-[#333] text-white"
        >
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>

      <div className="text-sm text-[#777]">
        第 {currentPage} 页，共 {totalPages} 页
      </div>
    </div>
  )
}

