import type React from "react"
import { Inter } from "next/font/google"
import { ThemeProvider } from "@/components/theme-provider"
import { Toaster } from "@/components/ui/toaster"
import { Sidebar } from "@/components/sidebar"
import { ErrorProvider } from "@/components/providers/error-provider"
import { QueryProvider } from "@/components/providers/query-provider"
import "./globals.css"

const inter = Inter({ subsets: ["latin"] })

export const metadata = {
  title: "动漫匹配系统 | Anime Matcher",
  description: "管理员核对来自TMDB和BgmTV的动漫数据匹配结果",
    generator: 'v0.dev'
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <body className={`${inter.className} dark`}>
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false} disableTransitionOnChange>
          <QueryProvider>
            <ErrorProvider>
              <div className="flex h-screen bg-[#0a0a0a] overflow-hidden">
                <Sidebar />
                <div className="flex-1 overflow-auto page-transition-container">{children}</div>
              </div>
              <Toaster />
            </ErrorProvider>
          </QueryProvider>
        </ThemeProvider>
      </body>
    </html>
  )
}



import './globals.css'