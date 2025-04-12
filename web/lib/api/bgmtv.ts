// Bangumi (bgm.tv) API documentation: https://github.com/bangumi/api

const BGMTV_BASE_URL = "https://api.bgm.tv"

export interface BGMTVImage {
  large: string
  common: string
  medium: string
  small: string
  grid: string
}

export interface BGMTVRating {
  total: number
  score: number
  count: {
    [key: string]: number
  }
}

export interface BGMTVCollection {
  doing: number
  collect: number
  wish: number
  dropped: number
  on_hold: number
}

export interface BGMTVSearchResult {
  id: number
  name: string
  name_cn: string
  type: number // 1: Book, 2: Anime, 3: Music, 4: Game, 6: Real
  summary: string
  air_date: string
  air_weekday: number
  images: BGMTVImage
  eps: number
  eps_count: number
  rating: BGMTVRating
  rank: number
  collection: BGMTVCollection
}

export interface BGMTVSearchResponse {
  results: number
  list: BGMTVSearchResult[]
}

export interface BGMTVCharacterInfo {
  gender: string
  birth_year: number
  birth_mon: number
  birth_day: number
  height: number
  weight: number
  blood_type: string
  bwh: string
  source: string
}

export interface BGMTVActorImage {
  large: string
  medium: string
  small: string
  grid: string
}

export interface BGMTVActor {
  id: number
  name: string
  name_cn: string
  images: BGMTVActorImage
}

export interface BGMTVCharacter {
  id: number
  name: string
  name_cn: string
  role_name: string
  role_name_cn: string
  images: BGMTVActorImage
  comment: number
  collects: number
  info: BGMTVCharacterInfo
  actors: BGMTVActor[]
}

export interface BGMTVStaff {
  id: number
  name: string
  name_cn: string
  role_name: string
  images: BGMTVActorImage
  comment: number
  collects: number
  info: BGMTVCharacterInfo
  jobs: string[]
  role_name_cn: string
}

export interface BGMTVTag {
  name: string
  count: number
}

export interface BGMTVInfoboxItem {
  key: string
  value: string | Array<{ v: string }>
}

export interface BGMTVAnimeDetail {
  id: number
  name: string
  name_cn: string
  type: number
  summary: string
  date: string
  air_weekday: number
  images: BGMTVImage
  eps: number
  eps_count: number
  rating: BGMTVRating
  rank: number
  collection: BGMTVCollection
  tags: BGMTVTag[]
  infobox: BGMTVInfoboxItem[]
  locked: boolean
  nsfw: boolean
  characters: BGMTVCharacter[]
  staff: BGMTVStaff[]
}

export async function searchBGMTVAnime(query: string): Promise<BGMTVSearchResponse> {
  const url = new URL(`${BGMTV_BASE_URL}/search/subject/${encodeURIComponent(query)}`)
  url.searchParams.append("type", "2") // Type 2 is for anime
  url.searchParams.append("responseGroup", "small")
  url.searchParams.append("start", "0")
  url.searchParams.append("max_results", "10")

  const response = await fetch(url.toString(), {
    headers: {
      Accept: "application/json",
      "User-Agent": "AnimeMatcherApp/1.0",
    },
  })

  if (!response.ok) {
    throw new Error(`BGMTV API error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}

export async function getBGMTVAnimeDetail(id: string | number): Promise<BGMTVAnimeDetail> {
  const url = `${BGMTV_BASE_URL}/v0/subjects/${id}?responseGroup=large`

  const response = await fetch(url, {
    headers: {
      Accept: "application/json",
      "User-Agent": "AnimeMatcherApp/1.0",
    },
  })

  if (!response.ok) {
    throw new Error(`BGMTV API error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}

export function getBGMTVImageUrl(path: string | null): string {
  if (!path) {
    return ""
  }
  return path
}

// Helper function to extract specific info from BGMTV infobox
export function extractInfoboxValue(infobox: BGMTVAnimeDetail["infobox"], key: string): string {
  const item = infobox.find((item) => item.key === key)
  if (!item) return ""

  if (typeof item.value === "string") {
    return item.value
  } else if (Array.isArray(item.value)) {
    return item.value.map((v) => v.v).join(", ")
  }

  return ""
}

