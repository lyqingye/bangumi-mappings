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

export enum SubjectType {
  Book = 1,
  Anime = 2,
  Music = 3,
  Game = 4,
  Real = 6
}

export interface BGMTVSearchFilter {
  type?: SubjectType[]
  tag?: string[]
  meta_tags?: string[]
  air_date?: string[]
  rating?: string[]
  rank?: string[]
  nsfw?: boolean
}

export interface BGMTVSearchParams {
  keyword: string
  sort?: 'match' | 'heat' | 'rank' | 'score'
  filter?: BGMTVSearchFilter
}

export interface BGMTVSearchResponse {
  total: number
  limit: number
  offset: number
  data: BGMTVAnimeDetail[]
}

export async function searchBGMTV(
  params: BGMTVSearchParams = {
    keyword: '',
    sort: 'rank',
    filter: {
      type: [SubjectType.Anime],
    },
  },
  limit: number = 10,
  offset: number = 0,
): Promise<BGMTVSearchResponse> {
  const url = `${BGMTV_BASE_URL}/v0/search/subjects?limit=${limit}&offset=${offset}`

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json',
      'User-Agent': 'AnimeMatcherApp/1.0',
    },
    body: JSON.stringify(params)
  })

  if (!response.ok) {
    throw new Error(`BGMTV API error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}