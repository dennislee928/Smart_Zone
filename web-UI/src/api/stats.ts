import { useQuery } from '@tanstack/react-query'
import { apiClient } from './client'
import type { Stats } from '../types'

// Fetch stats
export function useStats() {
  return useQuery({
    queryKey: ['stats'],
    queryFn: async () => {
      const { data } = await apiClient.get<{ stats: Stats }>('/api/stats')
      return data.stats
    },
    refetchInterval: 30000, // Refetch every 30 seconds
  })
}
