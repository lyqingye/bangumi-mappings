// TMDB API documentation: https://developer.themoviedb.org/reference/intro/getting-started

const TMDB_API_KEY = process.env.TMDB_API_KEY || "" // Replace with your actual API key
const TMDB_BASE_URL = "https://api.themoviedb.org/3"

export interface TMDBSearchResult {
  id: number
  name: string
  original_name: string
  overview: string
  first_air_date: string
  poster_path: string | null
  backdrop_path: string | null
  vote_average: number
  vote_count: number
  popularity: number
  original_language: string
  genre_ids: number[]
  origin_country: string[]
}

export interface TMDBSearchResponse {
  page: number
  results: TMDBSearchResult[]
  total_results: number
  total_pages: number
}

export interface TMDBGenre {
  id: number
  name: string
}

export interface TMDBNetwork {
  id: number
  name: string
  logo_path: string | null
  origin_country: string
}

export interface TMDBProductionCompany {
  id: number
  name: string
  logo_path: string | null
  origin_country: string
}

export interface TMDBSeason {
  air_date: string
  episode_count: number
  id: number
  name: string
  overview: string
  poster_path: string | null
  season_number: number
}

export interface TMDBAnimeDetail {
  id: number
  name: string
  original_name: string
  overview: string
  first_air_date: string
  last_air_date: string
  poster_path: string | null
  backdrop_path: string | null
  vote_average: number
  vote_count: number
  popularity: number
  status: string
  number_of_episodes: number
  number_of_seasons: number
  genres: TMDBGenre[]
  networks: TMDBNetwork[]
  production_companies: TMDBProductionCompany[]
  seasons: TMDBSeason[]
  episode_run_time: number[]
  languages: string[]
  in_production: boolean
  type: string
}

export interface TMDBCast {
  id: number
  name: string
  character: string
  profile_path: string | null
  order: number
}

export interface TMDBCrew {
  id: number
  name: string
  department: string
  job: string
  profile_path: string | null
}

export interface TMDBCredits {
  cast: TMDBCast[]
  crew: TMDBCrew[]
}

export interface TMDBResponse {
  detail: TMDBAnimeDetail
  credits: TMDBCredits
}

export async function searchTMDBAnime(query: string): Promise<TMDBSearchResponse> {
  const url = new URL(`${TMDB_BASE_URL}/search/tv`)
  url.searchParams.append("api_key", TMDB_API_KEY)
  url.searchParams.append("query", query)
  url.searchParams.append("language", "zh-CN") // Chinese language results

  const response = await fetch(url.toString())

  if (!response.ok) {
    throw new Error(`TMDB API error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}

export async function getTMDBAnimeDetail(id: string | number): Promise<TMDBAnimeDetail> {
  const url = new URL(`${TMDB_BASE_URL}/tv/${id}`)
  url.searchParams.append("api_key", TMDB_API_KEY)
  url.searchParams.append("language", "zh-CN")
  url.searchParams.append("append_to_response", "credits")

  const response = await fetch(url.toString())

  if (!response.ok) {
    throw new Error(`TMDB API error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}

export async function getTMDBAnimeCredits(id: string | number): Promise<TMDBCredits> {
  const url = new URL(`${TMDB_BASE_URL}/tv/${id}/credits`)
  url.searchParams.append("api_key", TMDB_API_KEY)
  url.searchParams.append("language", "zh-CN")

  const response = await fetch(url.toString())

  if (!response.ok) {
    throw new Error(`TMDB API error: ${response.status} ${response.statusText}`)
  }

  return response.json()
}

export function getTMDBImageUrl(path: string | null, size = "original"): string {
  if (!path) {
    return ""
  }
  return `https://image.tmdb.org/t/p/${size}${path}`
}

