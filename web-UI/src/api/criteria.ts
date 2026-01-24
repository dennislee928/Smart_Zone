import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'
import type { Criteria, ApiResponse } from '../types'

// Fetch criteria
export function useCriteria() {
  return useQuery({
    queryKey: ['criteria'],
    queryFn: async () => {
      const { data } = await apiClient.get<ApiResponse<Criteria>>('/api/criteria')
      return data.criteria as Criteria | null
    },
  })
}

// Update criteria
export function useUpdateCriteria() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async (criteria: Omit<Criteria, 'id' | 'updatedAt'>) => {
      const { data } = await apiClient.put<ApiResponse<Criteria>>('/api/criteria', criteria)
      return data.criteria as Criteria
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['criteria'] })
    },
  })
}
