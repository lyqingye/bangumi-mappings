// AniList API documentation: https://anilist.gitbook.io/anilist-apiv2-docs/

// AniList uses GraphQL, so we'll need to make POST requests with queries
const ANILIST_API_URL = "https://graphql.anilist.co"

export interface AniListTitle {
  romaji: string
  english: string | null
  native: string
  userPreferred: string
}

export interface AniListCoverImage {
  large: string
  medium: string
  color: string | null
}

export interface AniListTag {
  id: number
  name: string
  category: string
  rank: number
}

export interface AniListStudioNode {
  id: number
  name: string
  isAnimationStudio: boolean
}

export interface AniListStudios {
  nodes: AniListStudioNode[]
}

export interface AniListCharacterName {
  full: string
  native: string | null
}

export interface AniListCharacterImage {
  large: string
  medium: string
}

export interface AniListCharacterNode {
  id: number
  name: AniListCharacterName
  image: AniListCharacterImage
  description: string | null
  gender: string | null
}

export interface AniListCharacterEdge {
  node: AniListCharacterNode
  role: string
}

export interface AniListCharacters {
  edges: AniListCharacterEdge[]
}

export interface AniListStaffName {
  full: string
  native: string | null
}

export interface AniListStaffImage {
  large: string
  medium: string
}

export interface AniListStaffNode {
  id: number
  name: AniListStaffName
  image: AniListStaffImage
  description: string | null
  primaryOccupations: string[]
}

export interface AniListStaffEdge {
  node: AniListStaffNode
  role: string
}

export interface AniListStaff {
  edges: AniListStaffEdge[]
}

export interface AniListDate {
  year: number | null
  month: number | null
  day: number | null
}

export interface AniListAnimeDetail {
  id: number
  title: AniListTitle
  description: string | null
  coverImage: AniListCoverImage
  bannerImage: string | null
  season: string | null
  seasonYear: number | null
  format: string | null
  status: string | null
  episodes: number | null
  duration: number | null
  genres: string[]
  tags: AniListTag[]
  averageScore: number | null
  meanScore: number | null
  popularity: number | null
  studios: AniListStudios
  characters: AniListCharacters
  staff: AniListStaff
  startDate: AniListDate
  endDate: AniListDate
  source: string | null
}

// GraphQL query for fetching anime details - Fixed to use correct field structure
const ANIME_DETAIL_QUERY = `
query ($id: Int) {
  Media(id: $id, type: ANIME) {
    id
    title {
      romaji
      english
      native
      userPreferred
    }
    description(asHtml: false)
    coverImage {
      large
      medium
      color
    }
    bannerImage
    season
    seasonYear
    format
    status
    episodes
    duration
    genres
    tags {
      id
      name
      category
      rank
    }
    averageScore
    meanScore
    popularity
    studios {
      nodes {
        id
        name
        isAnimationStudio
      }
    }
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    source
  }
}
`

export async function getAniListAnimeDetail(id: string | number): Promise<AniListAnimeDetail> {
  let retries = 1

  while (retries > 0) {
    try {
      const response = await fetch(ANILIST_API_URL, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify({
          query: ANIME_DETAIL_QUERY,
          variables: {
            id: Number(id),
          },
        }),
      })

      if (!response.ok) {
        const errorText = await response.text()
        console.error(`AniList API error response (${response.status}):`, errorText)
        throw new Error(`AniList API error: ${response.status} ${response.statusText}`)
      }

      const json = await response.json()

      if (json.errors) {
        console.error("AniList GraphQL errors:", json.errors)
        throw new Error(`AniList API error: ${json.errors[0].message}`)
      }

      if (!json.data || !json.data.Media) {
        console.error("AniList API returned no data:", json)
        throw new Error("AniList API returned no data")
      }

      return json.data.Media
    } catch (error) {
      retries--
      if (retries === 0) {
        console.error("Error fetching from AniList after all retries:", error)

        // 返回一个模拟的AniList数据，以防API调用完全失败
        // 这样至少可以显示一些内容而不是完全崩溃
        return {
          id: Number(id),
          title: {
            romaji: "Unknown Anime",
            english: "Unknown Anime",
            native: "不明なアニメ",
            userPreferred: "Unknown Anime",
          },
          description: "Could not load anime details from AniList.",
          coverImage: {
            large: "/placeholder.svg?height=400&width=300&text=AniList+Error",
            medium: "/placeholder.svg?height=300&width=200&text=AniList+Error",
            color: "#8A4FFF",
          },
          bannerImage: null,
          season: null,
          seasonYear: new Date().getFullYear(),
          format: "TV",
          status: "UNKNOWN",
          episodes: 0,
          duration: 0,
          genres: [],
          tags: [],
          averageScore: 0,
          meanScore: 0,
          popularity: 0,
          studios: { nodes: [] },
          characters: { edges: [] },
          staff: { edges: [] },
          startDate: { year: null, month: null, day: null },
          endDate: { year: null, month: null, day: null },
          source: null,
        }
      }

      // 如果还有重试次数，等待一秒后重试
      console.warn(`Error fetching from AniList (retries left: ${retries}):`, error)
      await new Promise((resolve) => setTimeout(resolve, 1000))
    }
  }

  // TypeScript需要这个返回语句，但实际上代码不会执行到这里
  throw new Error("Failed to fetch AniList data after all retries")
}

export function getAniListImageUrl(path: string | null): string {
  if (!path) {
    return ""
  }
  return path
}

