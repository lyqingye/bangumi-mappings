"use client"

import { motion } from "framer-motion"
import Link from "next/link"
import { Badge } from "@/components/ui/badge"
import { cardVariants } from "@/animations/variants"
import { hoverTransition } from "@/animations/transitions"
import type { AnimeMapping, VerificationStatus } from "@/lib/types"
import { useAniListAnimeDetail } from "@/lib/api/hooks"
import Image from "next/image"
import { getStatusLabel } from "@/lib/utils"

interface MappingCardProps {
  mapping: AnimeMapping
  index: number
}

export function MappingCard({ mapping, index }: MappingCardProps) {
  // 使用 AniList ID 获取详细信息
  const { data: anilistData, isLoading, isError } = useAniListAnimeDetail(mapping.anilist_id)

  // 修改getVerificationBadge函数，使用getStatusLabel
  const getVerificationBadge = (status: VerificationStatus) => {
    const { label, color } = getStatusLabel(status)
    return <Badge className={`${color} text-white select-none text-xs`}>{label}</Badge>
  }

  // 加载状态
  if (isLoading) {
    return (
      <motion.div
        className="h-full"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.5 }}
        custom={index}
        variants={cardVariants}
      >
        <div className="bg-[#111] border border-[#222] rounded-lg overflow-hidden h-full flex flex-col">
          <div className="relative">
            <div className="grid grid-cols-1 gap-1 p-2">
              <div className="aspect-[3/4] rounded-md w-full overflow-hidden relative">
                <div className="absolute inset-0 bg-gradient-to-br from-[#222] to-[#333] animate-pulse" />
              </div>
            </div>
          </div>
          <div className="p-3 flex-1 flex flex-col">
            <div className="h-4 w-3/4 mb-2 bg-[#222] rounded-md animate-pulse" />
            <div className="h-3 w-1/2 mb-3 bg-[#222] rounded-md animate-pulse" />
            <div className="flex gap-2 mb-3">
              <div className="h-4 w-16 bg-[#222] rounded-md animate-pulse" />
              <div className="h-4 w-16 bg-[#222] rounded-md animate-pulse" />
            </div>
          </div>
        </div>
      </motion.div>
    )
  }

  // 错误状态
  if (isError || !anilistData) {
    return (
      <motion.div
        className="h-full"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.5 }}
        custom={index}
        variants={cardVariants}
      >
        <div className="bg-[#111] border border-[#222] rounded-lg overflow-hidden h-full flex flex-col p-4">
          <div className="text-center text-[#777] flex flex-col items-center justify-center h-full">
            <div className="text-sm mb-2">加载失败</div>
            <div className="text-xs">无法获取动漫数据</div>
          </div>
        </div>
      </motion.div>
    )
  }

  // 从 AniList 数据中提取信息
  const title = anilistData.title?.english || anilistData.title?.romaji || "未知标题"
  const originalTitle = anilistData.title?.native || ""
  const coverImage = anilistData.coverImage?.large || ""
  const year =
    anilistData.seasonYear || (anilistData.startDate?.year ? anilistData.startDate.year : new Date().getFullYear())

  return (
    <motion.div
      className="h-full"
      initial="hidden"
      animate="visible"
      exit="exit"
      custom={index}
      variants={cardVariants}
    >
      <Link href={`/anime/${mapping.anilist_id}`} prefetch={true} scroll={false}>
        <motion.div
          className="bg-[#111] border border-[#222] rounded-lg overflow-hidden hover:border-[#444] transition-colors h-full flex flex-col"
          whileHover={{
            scale: 1.02,
            boxShadow: "0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)",
            transition: hoverTransition,
          }}
        >
          {/* 卡片头部 */}
          <div className="relative">
            <div className="grid grid-cols-1 gap-1 p-2">
              <motion.div
                className="relative aspect-[3/4] rounded-md overflow-hidden bg-[#222]"
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.5 }}
              >
                <Image
                  src={coverImage || "/placeholder.svg"}
                  alt={title}
                  fill
                  className="object-cover"
                  sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
                  priority={index < 4}
                />
                <div className="absolute top-0 left-0 bg-purple-600 text-white text-xs px-1.5 py-0.5 rounded-br-md">
                  AniList
                </div>
              </motion.div>
            </div>

            {/* 匹配计数指示器 */}
            <motion.div
              className="absolute top-2 right-2 flex items-center justify-center w-8 h-8 rounded-full bg-black/70 backdrop-blur-sm"
              initial={{ scale: 0, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              transition={{
                delay: 0.2,
                duration: 0.4,
                type: "spring",
                stiffness: 200,
                damping: 15,
              }}
            >
              <div className="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold bg-blue-600">
                {mapping.match_count}
              </div>
            </motion.div>
          </div>

          {/* 卡片内容 */}
          <div className="p-3 flex-1 flex flex-col">
            <h3 className="font-bold text-sm mb-1 line-clamp-1">{title}</h3>
            <p className="text-xs text-[#777] mb-2 line-clamp-1">{originalTitle}</p>

            <div className="flex gap-2 text-xs text-[#777] mb-3">
              <span>{year}</span>
              <span>•</span>
              <span>ID: {mapping.anilist_id}</span>
            </div>

            {/* 验证状态 */}
            <div className="flex flex-wrap gap-1 mb-3">
              <div className="flex items-center gap-1">
                <span className="text-xs text-[#777]">BgmTV:</span>
                {getVerificationBadge(mapping.bgm_tv_verify_status)}
              </div>
              <div className="flex items-center gap-1">
                <span className="text-xs text-[#777]">TMDB:</span>
                {getVerificationBadge(mapping.tmdb_verify_status)}
              </div>
            </div>
          </div>
        </motion.div>
      </Link>
    </motion.div>
  )
}

