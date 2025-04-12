"use client"

import { createContext, useContext, useState, type ReactNode, useEffect } from "react"
import { AlertCircle, X } from "lucide-react"
import { cn } from "@/lib/utils"
import { useToast } from "@/components/ui/use-toast"

type ErrorContextType = {
  setError: (message: string) => void
  clearError: () => void
}

const ErrorContext = createContext<ErrorContextType | undefined>(undefined)

export function ErrorProvider({ children }: { children: ReactNode }) {
  const [error, setErrorMessage] = useState<string | null>(null)
  const { toast } = useToast()

  const setError = (message: string) => {
    // 如果是相同的错误消息，不重复设置
    if (error === message) return

    setErrorMessage(message)
    toast({
      variant: "destructive",
      title: "错误",
      description: message,
      duration: 5000,
    })
  }

  const clearError = () => {
    setErrorMessage(null)
  }

  useEffect(() => {
    // 如果有错误消息，设置一个定时器在10秒后自动清除
    if (error) {
      const timer = setTimeout(() => {
        clearError()
      }, 10000)
      return () => clearTimeout(timer)
    }
  }, [error])

  return (
    <ErrorContext.Provider value={{ setError, clearError }}>
      {children}
      {error && (
        <div
          className={cn(
            "fixed right-4 top-4 z-50 flex w-96 items-center gap-2 rounded-lg bg-destructive p-4 text-destructive-foreground shadow-lg",
            "animate-in slide-in-from-top-5",
          )}
        >
          <AlertCircle className="h-5 w-5" />
          <div className="flex-1">{error}</div>
          <button onClick={clearError} className="rounded-full p-1 hover:bg-background/20">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}
    </ErrorContext.Provider>
  )
}

export function useError() {
  const context = useContext(ErrorContext)
  if (context === undefined) {
    throw new Error("useError must be used within an ErrorProvider")
  }
  return context
}

