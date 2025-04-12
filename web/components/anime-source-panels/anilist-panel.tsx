"use client"

import { motion } from "framer-motion"
import { useState } from "react"
import Image from "next/image"
import Link from "next/link"
import { Badge } from "@/components/ui/badge"
import { ChevronDown, ExternalLink } from "lucide-react"
import type { AniListAnimeDetail } from "@/lib/api/anilist"
import { formatDate } from "@/lib/utils"

interface AniListPanelProps {
  data: AniListAnimeDetail | null
  delay?: number
}

export function AniListPanel({ data, delay = 0.2 }: AniListPanelProps) {
  const [overviewExpanded, setOverviewExpanded] = useState(false)

  if (!data) return null

  // Extract data directly from the API response
  const id = data.id
  const title = data.title?.english || data.title?.romaji || "Unknown Title"
  const originalTitle = data.title?.native || ""
  const coverImage = data.coverImage?.large || ""
  const season = data.season || ""
  const year = data.seasonYear || 0
  const episodes = data.episodes || 0
  const status = data.status || "Unknown"
  const score = data.averageScore ? (data.averageScore / 10).toFixed(1) : "N/A"
  const genres = data.genres || []

  // 格式化开始日期
  const startDate = data.startDate?.year
    ? formatDate(`${data.startDate.year}-${data.startDate.month || 1}-${data.startDate.day || 1}`)
    : "Unknown"

  return (
    <motion.div
      className="bg-[#111] border border-[#222] rounded-lg overflow-hidden h-[450px] flex flex-col"
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay }}
    >
      <div className="p-4 bg-purple-900/20 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Badge className="bg-purple-600 text-white">AniList</Badge>
          <span className="font-semibold">Source Data</span>
        </div>
        <Link
          href={`https://anilist.co/anime/${id}`}
          target="_blank"
          className="text-purple-400 hover:text-purple-300 text-sm flex items-center gap-1"
        >
          View on AniList <ExternalLink className="h-3 w-3" />
        </Link>
      </div>

      <div className="p-4 flex-1 overflow-y-auto scrollbar-hide">
        <div className="flex gap-4">
          <div className="w-24 h-36 bg-[#222] rounded-md relative flex-shrink-0">
            <Image
              src={coverImage || "/placeholder.svg"}
              alt={title}
              fill
              className="object-cover rounded-md"
              sizes="96px"
            />
          </div>

          <div className="flex-1">
            <h3 className="text-xl font-bold mb-1">
              {title}
            </h3>
            <p className="text-[#777] text-sm mb-4">
              {originalTitle}
            </p>

            <div
              className="flex items-center justify-between mb-2 cursor-pointer"
              onClick={() => setOverviewExpanded((prev) => !prev)}
            >
              <span className="text-sm font-medium">Overview</span>
              <ChevronDown
                className={`h-4 w-4 text-[#777] transition-transform ${overviewExpanded ? "rotate-180" : ""}`}
              />
            </div>

            {overviewExpanded && (
              <motion.div
                className="mb-4 text-sm text-[#777]"
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.3 }}
              >
                {data.description || "No description available."}
              </motion.div>
            )}

            <div className="grid grid-cols-2 gap-x-4 gap-y-2 mb-4">
              <div>
                <div className="text-xs text-[#777]">Season</div>
                <div className="text-sm">
                  {season} {year}
                </div>
              </div>
              <div>
                <div className="text-xs text-[#777]">Episodes</div>
                <div className="text-sm">{episodes}</div>
              </div>
              <div>
                <div className="text-xs text-[#777]">Status</div>
                <div className="text-sm">{status}</div>
              </div>
              <div>
                <div className="text-xs text-[#777]">Rating</div>
                <div className="text-sm">{score} / 10</div>
              </div>
            </div>

            <div>
              <div className="text-xs text-[#777] mb-2">Genres</div>
              <div className="flex flex-wrap gap-1">
                {genres.map((genre: string, index: number) => (
                  <Badge
                    key={`anilist-genre-${index}`}
                    variant="secondary"
                    className="bg-[#222] text-white border-none text-xs"
                  >
                    {genre}
                  </Badge>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  )
}

