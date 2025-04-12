"use client"

import { useState, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useQueryClient } from "@tanstack/react-query"
import { ChevronLeft, ChevronRight, RefreshCw, Search, Play, Plus, Pause, RotateCcw, Trash2, Download, Upload, Minimize, BarChart } from "lucide-react"

// 组件导入
import { AnimeCard } from "@/components/anime-card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { PageTransition } from "@/components/page-transition"
import { useError } from "@/components/providers/error-provider"

// 钩子和工具导入
import { containerVariants, itemVariants, statsVariants, statItemVariants } from "@/animations/variants"
import { hoverTransition } from "@/animations/transitions"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { 
  createJob, 
  getSummary, 
  getYearStatistics,
  listJobs, 
  runJob, 
  pauseJob, 
  resumeJob, 
  removeJob, 
  exportAnimes, 
  importAnimes, 
  compactAnimes 
} from "@/lib/api/animes"
import { 
  JobDetails, 
  Platform, 
  Provider, 
  ProviderModelMap, 
  JobStatus, 
  Summary, 
  YearStatistic,
  YearStatistics
} from "@/lib/types"
import { 
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"

// 在组件内，statsData下方添加一个常量
export default function Home() {
  const [jobs, setJobs] = useState<JobDetails[]>([])
  const [summary, setSummary] = useState<Summary | null>(null)
  const [yearStats, setYearStats] = useState<YearStatistic[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [isSummaryLoading, setIsSummaryLoading] = useState(false)
  const [isYearStatsLoading, setIsYearStatsLoading] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [jobToRemove, setJobToRemove] = useState<{platform: Platform, year: number} | null>(null)
  const [jobForm, setJobForm] = useState({
    platform: Platform.BgmTv,
    year: new Date().getFullYear(),
    provider: Provider.Deepseek,
  })
  
  // 数据导出/导入状态
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false)
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false)
  const [dataYear, setDataYear] = useState<number>(new Date().getFullYear())
  const [dataProcessing, setDataProcessing] = useState(false)
  
  // 统计数据显示状态
  const [statsView, setStatsView] = useState<"summary" | "yearly">("summary")

  // 用于表格的布局常量
  const TABLE_ROW_HEIGHT = "64px" // 行高固定
  const TABLE_MIN_HEIGHT = "50px" // 表格最小高度

  // 获取统计数据
  const fetchSummary = async () => {
    setIsSummaryLoading(true)
    try {
      const data = await getSummary()
      setSummary(data)
    } catch (error) {
      console.error("Failed to fetch summary:", error)
    } finally {
      setIsSummaryLoading(false)
    }
  }
  
  // 获取年份统计数据
  const fetchYearStatistics = async () => {
    setIsYearStatsLoading(true)
    try {
      const data = await getYearStatistics()
      setYearStats(data.statistics)
    } catch (error) {
      console.error("Failed to fetch year statistics:", error)
    } finally {
      setIsYearStatsLoading(false)
    }
  }

  // 获取任务列表
  const fetchJobs = async () => {
    setIsLoading(true)
    try {
      const data = await listJobs()
      setJobs(data)
    } catch (error) {
      console.error("Failed to fetch jobs:", error)
    } finally {
      setIsLoading(false)
    }
  }

  // 创建任务
  const handleCreateJob = async () => {
    try {
      await createJob(
        jobForm.platform, 
        jobForm.year, 
        jobForm.provider, 
        ProviderModelMap[jobForm.provider]
      )
      setIsCreateDialogOpen(false)
      fetchJobs()
    } catch (error) {
      console.error("Failed to create job:", error)
    }
  }

  // 运行任务
  const handleRunJob = async (platform: Platform, year: number) => {
    try {
      await runJob(platform, year)
      fetchJobs()
    } catch (error) {
      console.error("Failed to run job:", error)
    }
  }

  // 暂停任务
  const handlePauseJob = async (platform: Platform, year: number) => {
    try {
      await pauseJob(platform, year)
      fetchJobs()
    } catch (error) {
      console.error("Failed to pause job:", error)
    }
  }

  // 恢复任务
  const handleResumeJob = async (platform: Platform, year: number) => {
    try {
      await resumeJob(platform, year)
      fetchJobs()
    } catch (error) {
      console.error("Failed to resume job:", error)
    }
  }

  // 移除任务
  const handleRemoveJob = async () => {
    if (!jobToRemove) return;
    
    try {
      await removeJob(jobToRemove.platform, jobToRemove.year)
      setJobToRemove(null)
      fetchJobs()
    } catch (error) {
      console.error("Failed to remove job:", error)
    }
  }

  // 获取任务状态标签样式
  const getStatusStyle = (status: JobStatus) => {
    switch (status) {
      case JobStatus.Running:
        return 'bg-blue-900/30 text-blue-300';
      case JobStatus.Paused:
        return 'bg-yellow-900/30 text-yellow-300';
      case JobStatus.Completed:
        return 'bg-green-900/30 text-green-300';
      case JobStatus.Failed:
        return 'bg-red-900/30 text-red-300';
      default:
        return 'bg-gray-900/30 text-gray-300';
    }
  }

  // 获取任务状态文本
  const getStatusText = (status: JobStatus) => {
    switch (status) {
      case JobStatus.Created:
        return '等待中';
      case JobStatus.Running:
        return '运行中';
      case JobStatus.Paused:
        return '已暂停';
      case JobStatus.Completed:
        return '已完成';
      case JobStatus.Failed:
        return '失败';
      default:
        return '未知';
    }
  }

  // 导出数据
  const handleExportData = async () => {
    try {
      setDataProcessing(true)
      await exportAnimes(dataYear)
      setIsExportDialogOpen(false)
      alert(`成功导出${dataYear}年数据`)
      // 刷新统计数据
      fetchSummary()
    } catch (error) {
      console.error("导出数据失败:", error)
      alert(`导出数据失败: ${error}`)
    } finally {
      setDataProcessing(false)
    }
  }

  // 导入数据
  const handleImportData = async () => {
    try {
      setDataProcessing(true)
      await importAnimes(dataYear)
      setIsImportDialogOpen(false)
      alert(`成功导入${dataYear}年数据`)
      // 刷新统计数据
      fetchSummary()
    } catch (error) {
      console.error("导入数据失败:", error)
      alert(`导入数据失败: ${error}`)
    } finally {
      setDataProcessing(false)
    }
  }

  // 压缩数据
  const handleCompactData = async () => {
    try {
      setDataProcessing(true)
      await compactAnimes()
      alert('成功压缩数据')
    } catch (error) {
      console.error("压缩数据失败:", error)
      alert(`压缩数据失败: ${error}`)
    } finally {
      setDataProcessing(false)
    }
  }

  // 初始加载任务列表和统计数据
  useEffect(() => {
    fetchJobs()
    fetchSummary()
    fetchYearStatistics()
    
    // 设置3秒刷新一次的定时器
    const intervalId = setInterval(() => {
      fetchJobs()
    }, 3000)
    
    // 组件卸载时清除定时器
    return () => clearInterval(intervalId)
  }, [])

  // 构建统计卡片数据
  const buildStatsData = () => {
    if (!summary) {
      return [
        { id: "total_animes", value: "-", label: "总动漫数", sublabel: "Database" },
        { id: "total_tmdb_matched", value: "-", label: "TMDB已匹配", sublabel: "Matched" },
        { id: "total_tmdb_unmatched", value: "-", label: "TMDB未匹配", sublabel: "Unmatched" },
        { id: "total_tmdb_dropped", value: "-", label: "TMDB已放弃", sublabel: "Dropped" },
        { id: "total_bgmtv_matched", value: "-", label: "BgmTV已匹配", sublabel: "Matched" },
        { id: "total_bgmtv_unmatched", value: "-", label: "BgmTV未匹配", sublabel: "Unmatched" },
        { id: "total_bgmtv_dropped", value: "-", label: "BgmTV已放弃", sublabel: "Dropped" },
      ]
    }

    return [
      { id: "total_animes", value: summary.total_animes.toLocaleString(), label: "总动漫数", sublabel: "Database" },
      { id: "total_tmdb_matched", value: summary.total_tmdb_matched.toLocaleString(), label: "TMDB已匹配", sublabel: "Matched" },
      { id: "total_tmdb_unmatched", value: summary.total_tmdb_unmatched.toLocaleString(), label: "TMDB未匹配", sublabel: "Unmatched" },
      { id: "total_tmdb_dropped", value: summary.total_tmdb_dropped.toLocaleString(), label: "TMDB已放弃", sublabel: "Dropped" },
      { id: "total_bgmtv_matched", value: summary.total_bgmtv_matched.toLocaleString(), label: "BgmTV已匹配", sublabel: "Matched" },
      { id: "total_bgmtv_unmatched", value: summary.total_bgmtv_unmatched.toLocaleString(), label: "BgmTV未匹配", sublabel: "Unmatched" },
      { id: "total_bgmtv_dropped", value: summary.total_bgmtv_dropped.toLocaleString(), label: "BgmTV已放弃", sublabel: "Dropped" },
    ]
  }

  const statsData = buildStatsData();

  // 获取进度百分比的计算函数
  const getCompletionPercent = (matched: number, unmatched: number, dropped: number, total: number) => {
    const processedCount = matched + dropped;
    return total > 0 ? Math.round((processedCount / total) * 100) : 0;
  }

  return (
    <PageTransition>
      <div className="min-h-screen bg-[#0a0a0a]">
        {/* Header */}
        <motion.div
          className="border-b border-[#222] bg-[#111] w-full"
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
        >
          <div className="w-full px-6 py-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <ChevronLeft className="h-5 w-5 text-[#777]" />
                <h1 className="text-xl font-bold">Anime Matcher</h1>
              </div>

              <div className="relative w-64">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[#777]" />
                <Input
                  placeholder="Search anime..."
                  className="pl-10 bg-[#222] border-[#333] text-white rounded-full"
                />
              </div>
            </div>
          </div>
        </motion.div>

        <div className="w-full px-6 py-6">
          {/* 统计视图切换标签 */}
          <div className="mb-4 flex justify-between items-center">
            <Tabs value={statsView} onValueChange={(v) => setStatsView(v as "summary" | "yearly")} className="w-[400px]">
              <TabsList className="bg-[#111] border border-[#333]">
                <TabsTrigger value="summary" className="data-[state=active]:bg-[#222]">
                  总体统计
                </TabsTrigger>
                <TabsTrigger value="yearly" className="data-[state=active]:bg-[#222]">
                  年份统计
                </TabsTrigger>
              </TabsList>
            </Tabs>
            
            <Button 
              variant="outline" 
              size="sm" 
              className="border-[#333] text-[#777] hover:text-white"
              onClick={() => {
                fetchSummary();
                fetchYearStatistics();
              }}
            >
              <RefreshCw className="mr-2 h-4 w-4" />
              刷新统计
            </Button>
          </div>
          
          {/* 统计视图内容 */}
          <AnimatePresence mode="wait">
            {statsView === "summary" ? (
              /* 总体统计卡片 */
              <motion.div
                key="summary-stats"
                className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-7 gap-4 mb-6"
                variants={statsVariants}
                initial="hidden"
                animate="visible"
                exit={{ opacity: 0, transition: { duration: 0.2 } }}
              >
                {statsData.map((stat) => (
                  <motion.div
                    key={stat.id}
                    className="bg-[#111] border border-[#222] rounded-lg p-4"
                    variants={statItemVariants}
                    whileHover={{
                      scale: 1.02,
                      boxShadow: "0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)",
                      transition: hoverTransition,
                    }}
                  >
                    <div className="text-3xl font-bold mb-1">{stat.value}</div>
                    <div className="text-sm text-[#777]">{stat.label}</div>
                    <div className="text-xs text-[#555]">{stat.sublabel}</div>
                  </motion.div>
                ))}
              </motion.div>
            ) : (
              /* 年份统计表格 */
              <motion.div
                key="yearly-stats"
                className="bg-[#111] border border-[#222] rounded-lg overflow-hidden mb-6"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20, transition: { duration: 0.2 } }}
                transition={{ duration: 0.3 }}
              >
                {isYearStatsLoading ? (
                  <div className="p-8 text-center text-[#777]">加载年份统计数据中...</div>
                ) : yearStats.length === 0 ? (
                  <div className="p-8 text-center text-[#777]">暂无年份统计数据</div>
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead>
                        <tr className="border-b border-[#222] bg-[#1a1a1a]">
                          <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase">年份</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase">总动漫数</th>
                          <th className="px-4 py-3 text-center text-xs font-medium text-[#777] uppercase" colSpan={4}>TMDB</th>
                          <th className="px-4 py-3 text-center text-xs font-medium text-[#777] uppercase" colSpan={4}>BgmTV</th>
                        </tr>
                        <tr className="border-b border-[#222] bg-[#1a1a1a]">
                          <th className="px-4 py-2"></th>
                          <th className="px-4 py-2"></th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">已匹配</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">未匹配</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">已放弃</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">完成率</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">已匹配</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">未匹配</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">已放弃</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-[#777]">完成率</th>
                        </tr>
                      </thead>
                      <tbody>
                        {yearStats.map((stat) => {
                          const tmdbCompletionPercent = getCompletionPercent(
                            stat.tmdb_matched, 
                            stat.tmdb_unmatched, 
                            stat.tmdb_dropped,
                            stat.total_animes
                          );
                          
                          const bgmtvCompletionPercent = getCompletionPercent(
                            stat.bgmtv_matched, 
                            stat.bgmtv_unmatched, 
                            stat.bgmtv_dropped,
                            stat.total_animes
                          );
                          
                          return (
                            <motion.tr 
                              key={stat.year}
                              className="border-b border-[#222] hover:bg-[#1a1a1a]"
                              initial={{ opacity: 0 }}
                              animate={{ opacity: 1 }}
                              transition={{ duration: 0.2 }}
                            >
                              <td className="px-4 py-3 text-sm font-medium">{stat.year}</td>
                              <td className="px-4 py-3 text-sm">{stat.total_animes}</td>
                              
                              {/* TMDB 统计 */}
                              <td className="px-4 py-3 text-sm text-green-400">{stat.tmdb_matched}</td>
                              <td className="px-4 py-3 text-sm text-yellow-400">{stat.tmdb_unmatched}</td>
                              <td className="px-4 py-3 text-sm text-red-400">{stat.tmdb_dropped}</td>
                              <td className="px-4 py-3 text-sm">
                                <div className="flex items-center space-x-2">
                                  <div className="w-full h-1.5 bg-[#222] rounded-full overflow-hidden">
                                    <div 
                                      className="h-full bg-blue-500 rounded-full" 
                                      style={{ width: `${tmdbCompletionPercent}%` }}
                                    ></div>
                                  </div>
                                  <span className="text-xs text-[#777]">{tmdbCompletionPercent}%</span>
                                </div>
                              </td>
                              
                              {/* BgmTV 统计 */}
                              <td className="px-4 py-3 text-sm text-green-400">{stat.bgmtv_matched}</td>
                              <td className="px-4 py-3 text-sm text-yellow-400">{stat.bgmtv_unmatched}</td>
                              <td className="px-4 py-3 text-sm text-red-400">{stat.bgmtv_dropped}</td>
                              <td className="px-4 py-3 text-sm">
                                <div className="flex items-center space-x-2">
                                  <div className="w-full h-1.5 bg-[#222] rounded-full overflow-hidden">
                                    <div 
                                      className="h-full bg-blue-500 rounded-full" 
                                      style={{ width: `${bgmtvCompletionPercent}%` }}
                                    ></div>
                                  </div>
                                  <span className="text-xs text-[#777]">{bgmtvCompletionPercent}%</span>
                                </div>
                              </td>
                            </motion.tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                )}
              </motion.div>
            )}
          </AnimatePresence>

          {/* 任务列表 */}
          <div className="mb-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-white">任务列表</h2>
              <div className="flex gap-2">
                {/* 数据操作按钮组 */}
                <div className="flex gap-2 mr-4">
                  <Dialog open={isExportDialogOpen} onOpenChange={setIsExportDialogOpen}>
                    <DialogTrigger asChild>
                      <Button 
                        variant="outline" 
                        size="sm" 
                        className="border-[#333] text-[#777] hover:text-white"
                        disabled={dataProcessing}
                      >
                        <Download className="mr-2 h-4 w-4" />
                        导出数据
                      </Button>
                    </DialogTrigger>
                    <DialogContent className="bg-[#111] border-[#333] text-white">
                      <DialogHeader>
                        <DialogTitle>导出年份数据</DialogTitle>
                      </DialogHeader>
                      <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                          <label className="text-sm text-[#777]">年份</label>
                          <Input 
                            type="number" 
                            value={dataYear} 
                            onChange={(e) => setDataYear(parseInt(e.target.value))}
                            className="bg-[#222] border-[#333] text-white"
                          />
                        </div>
                        
                        <Button 
                          onClick={handleExportData}
                          className="bg-[#333] hover:bg-[#444] text-white"
                          disabled={dataProcessing}
                        >
                          {dataProcessing ? '处理中...' : '导出'}
                        </Button>
                      </div>
                    </DialogContent>
                  </Dialog>
                  
                  <Dialog open={isImportDialogOpen} onOpenChange={setIsImportDialogOpen}>
                    <DialogTrigger asChild>
                      <Button 
                        variant="outline" 
                        size="sm" 
                        className="border-[#333] text-[#777] hover:text-white"
                        disabled={dataProcessing}
                      >
                        <Upload className="mr-2 h-4 w-4" />
                        导入数据
                      </Button>
                    </DialogTrigger>
                    <DialogContent className="bg-[#111] border-[#333] text-white">
                      <DialogHeader>
                        <DialogTitle>导入年份数据</DialogTitle>
                      </DialogHeader>
                      <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                          <label className="text-sm text-[#777]">年份</label>
                          <Input 
                            type="number" 
                            value={dataYear} 
                            onChange={(e) => setDataYear(parseInt(e.target.value))}
                            className="bg-[#222] border-[#333] text-white"
                          />
                        </div>
                        
                        <Button 
                          onClick={handleImportData}
                          className="bg-[#333] hover:bg-[#444] text-white"
                          disabled={dataProcessing}
                        >
                          {dataProcessing ? '处理中...' : '导入'}
                        </Button>
                      </div>
                    </DialogContent>
                  </Dialog>
                  
                  <Button 
                    variant="outline" 
                    size="sm" 
                    className="border-[#333] text-[#777] hover:text-white"
                    onClick={handleCompactData}
                    disabled={dataProcessing}
                  >
                    <Minimize className="mr-2 h-4 w-4" />
                    压缩数据
                  </Button>
                </div>
                
                <Button 
                  variant="outline" 
                  size="sm" 
                  className="border-[#333] text-[#777] hover:text-white"
                  onClick={() => {
                    fetchJobs();
                    fetchSummary();
                    fetchYearStatistics();
                  }}
                >
                  <RefreshCw className="mr-2 h-4 w-4" />
                  刷新
                </Button>
                
                <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
                  <DialogTrigger asChild>
                    <Button 
                      size="sm" 
                      className="bg-[#222] hover:bg-[#333] text-white"
                    >
                      <Plus className="mr-2 h-4 w-4" />
                      创建任务
                    </Button>
                  </DialogTrigger>
                  <DialogContent className="bg-[#111] border-[#333] text-white">
                    <DialogHeader>
                      <DialogTitle>创建新任务</DialogTitle>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <label className="text-sm text-[#777]">平台</label>
                        <Select 
                          value={jobForm.platform} 
                          onValueChange={(value) => setJobForm({...jobForm, platform: value as Platform})}
                        >
                          <SelectTrigger className="bg-[#222] border-[#333] text-white">
                            <SelectValue placeholder="选择平台" />
                          </SelectTrigger>
                          <SelectContent className="bg-[#222] border-[#333] text-white">
                            <SelectItem value={Platform.BgmTv}>BgmTV</SelectItem>
                            <SelectItem value={Platform.Tmdb}>TMDB</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      
                      <div className="grid gap-2">
                        <label className="text-sm text-[#777]">年份</label>
                        <Input 
                          type="number" 
                          value={jobForm.year} 
                          onChange={(e) => setJobForm({...jobForm, year: parseInt(e.target.value)})}
                          className="bg-[#222] border-[#333] text-white"
                        />
                      </div>
                      
                      <div className="grid gap-2">
                        <label className="text-sm text-[#777]">提供商</label>
                        <Select 
                          value={jobForm.provider} 
                          onValueChange={(value) => setJobForm({...jobForm, provider: value as Provider})}
                        >
                          <SelectTrigger className="bg-[#222] border-[#333] text-white">
                            <SelectValue placeholder="选择提供商" />
                          </SelectTrigger>
                          <SelectContent className="bg-[#222] border-[#333] text-white">
                            <SelectItem value={Provider.Xai}>Xai</SelectItem>
                            <SelectItem value={Provider.Deepseek}>Deepseek</SelectItem>
                            <SelectItem value={Provider.Gemini}>Gemini</SelectItem>
                            <SelectItem value={Provider.OpenAI}>OpenAI</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      
                      <Button 
                        onClick={handleCreateJob}
                        className="bg-[#333] hover:bg-[#444] text-white"
                      >
                        创建
                      </Button>
                    </div>
                  </DialogContent>
                </Dialog>
              </div>
            </div>
            
            {/* 移除任务确认对话框 */}
            <AlertDialog open={!!jobToRemove} onOpenChange={(open) => !open && setJobToRemove(null)}>
              <AlertDialogContent className="bg-[#111] border-[#333] text-white">
                <AlertDialogHeader>
                  <AlertDialogTitle>确认移除任务</AlertDialogTitle>
                  <AlertDialogDescription className="text-[#999]">
                    您确定要移除该任务吗？任务将先被暂停然后移除，此操作不可恢复。
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel className="bg-[#222] hover:bg-[#333] text-white border-[#444]">取消</AlertDialogCancel>
                  <AlertDialogAction 
                    className="bg-red-900/50 hover:bg-red-900/80 text-red-200"
                    onClick={handleRemoveJob}
                  >
                    移除
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
            
            <div className="bg-[#111] border border-[#222] rounded-lg overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full table-fixed" style={{ minHeight: TABLE_MIN_HEIGHT }}>
                  <thead>
                    <tr className="border-b border-[#222] bg-[#1a1a1a]">
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[10%]">平台</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[8%]">年份</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[12%]">提供商</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[15%]">模型</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[10%]">状态</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[25%]">进度</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[12%]">创建时间</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-[#777] uppercase w-[8%]">操作</th>
                    </tr>
                  </thead>
                  <tbody>
                    {isLoading && jobs.length === 0 ? (
                      <tr>
                        <td colSpan={8} className="px-4 py-6 text-center text-[#777]">加载中...</td>
                      </tr>
                    ) : jobs.length === 0 ? (
                      <tr>
                        <td colSpan={8} className="px-4 py-6 text-center text-[#777]">暂无任务</td>
                      </tr>
                    ) : (
                      <AnimatePresence>
                        {jobs.map((job) => {
                          // 使用平台、年份和提供商组合作为唯一稳定的key
                          const jobKey = `${job.platform}-${job.year}-${job.provider}`;
                          
                          return (
                            <motion.tr 
                              key={jobKey}
                              className="border-b border-[#222] hover:bg-[#1a1a1a]"
                              style={{ height: TABLE_ROW_HEIGHT }}
                              layout // 启用布局动画
                              layoutId={jobKey} // 为每行分配唯一的layoutId
                              transition={{
                                layout: { duration: 0.2, ease: "easeInOut" }
                              }}
                            >
                              <td className="px-4 py-3 text-sm">
                                <motion.div layout>{job.platform}</motion.div>
                              </td>
                              <td className="px-4 py-3 text-sm">
                                <motion.div layout>{job.year}</motion.div>
                              </td>
                              <td className="px-4 py-3 text-sm">
                                <motion.div layout>{job.provider}</motion.div>
                              </td>
                              <td className="px-4 py-3 text-sm">
                                <motion.div layout>{job.model}</motion.div>
                              </td>
                              <td className="px-4 py-3 text-sm">
                                <motion.span 
                                  layout
                                  className={`px-2 py-1 rounded-full text-xs ${getStatusStyle(job.status)}`}
                                >
                                  {getStatusText(job.status)}
                                </motion.span>
                              </td>
                              <td className="px-4 py-3 text-sm">
                                <motion.div layout className="space-y-1">
                                  <div className="w-full bg-[#222] rounded-full h-1.5">
                                    <motion.div 
                                      className="bg-blue-500 h-1.5 rounded-full"
                                      animate={{ 
                                        width: `${job.num_animes_to_match > 0 ? (job.num_processed / job.num_animes_to_match) * 100 : 0}%` 
                                      }}
                                      transition={{ duration: 0.3 }}
                                    ></motion.div>
                                  </div>
                                  <div className="text-xs text-[#777]">
                                    {job.num_processed}/{job.num_animes_to_match} 
                                    ({job.num_animes_to_match > 0 ? Math.round((job.num_processed / job.num_animes_to_match) * 100) : 0}%)
                                  </div>
                                  <div className="flex justify-between text-xs">
                                    <span className="text-green-400">成功: {job.num_matched}</span>
                                    <span className="text-red-400">失败: {job.num_failed}</span>
                                  </div>
                                </motion.div>
                              </td>
                              <td className="px-4 py-3 text-sm text-[#777]">
                                <motion.div layout>{new Date(job.job_start_time).toLocaleString('zh-CN')}</motion.div>
                              </td>
                              <td className="px-4 py-3 text-sm">
                                <motion.div layout className="flex space-x-1">
                                  {job.status === JobStatus.Created && (
                                    <Button 
                                      size="sm" 
                                      variant="ghost" 
                                      className="h-8 px-2 text-blue-400 hover:text-blue-300 hover:bg-blue-900/20"
                                      onClick={() => handleRunJob(job.platform, job.year)}
                                      title="运行"
                                    >
                                      <Play className="h-4 w-4" />
                                    </Button>
                                  )}
                                  
                                  {job.status === JobStatus.Running && (
                                    <Button 
                                      size="sm" 
                                      variant="ghost" 
                                      className="h-8 px-2 text-yellow-400 hover:text-yellow-300 hover:bg-yellow-900/20"
                                      onClick={() => handlePauseJob(job.platform, job.year)}
                                      title="暂停"
                                    >
                                      <Pause className="h-4 w-4" />
                                    </Button>
                                  )}
                                  
                                  {job.status === JobStatus.Paused && (
                                    <Button 
                                      size="sm" 
                                      variant="ghost" 
                                      className="h-8 px-2 text-green-400 hover:text-green-300 hover:bg-green-900/20"
                                      onClick={() => handleResumeJob(job.platform, job.year)}
                                      title="恢复"
                                    >
                                      <RotateCcw className="h-4 w-4" />
                                    </Button>
                                  )}
                                  
                                  {/* 移除任务按钮，所有状态下都可见 */}
                                  <Button 
                                    size="sm" 
                                    variant="ghost" 
                                    className="h-8 px-2 text-red-400 hover:text-red-300 hover:bg-red-900/20"
                                    onClick={() => setJobToRemove({platform: job.platform, year: job.year})}
                                    title="移除"
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                </motion.div>
                              </td>
                            </motion.tr>
                          );
                        })}
                      </AnimatePresence>
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      </div>
    </PageTransition>
  )
}

