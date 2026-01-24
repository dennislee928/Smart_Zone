import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'
import type { Application, ApiResponse } from '../types'

// Fetch all applications
export function useApplications() {
  return useQuery({
    queryKey: ['applications'],
    queryFn: async () => {
      const { data } = await apiClient.get<ApiResponse<Application>>('/api/applications')
      return (data.applications || []) as Application[]
    },
  })
}

// Fetch single application
export function useApplication(id: number) {
  return useQuery({
    queryKey: ['applications', id],
    queryFn: async () => {
      const { data } = await apiClient.get<ApiResponse<Application>>(`/api/applications/${id}`)
      return data.application as Application
    },
    enabled: !!id,
  })
}

// Create application
export function useCreateApplication() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async (application: Omit<Application, 'id' | 'createdAt' | 'updatedAt'>) => {
      const { data } = await apiClient.post<ApiResponse<Application>>('/api/applications', application)
      return data.application as Application
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications'] })
      queryClient.invalidateQueries({ queryKey: ['stats'] })
    },
  })
}

// Update application
export function useUpdateApplication() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async ({ id, ...updates }: Partial<Application> & { id: number }) => {
      const { data } = await apiClient.put<ApiResponse<Application>>(`/api/applications/${id}`, updates)
      return data.application as Application
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['applications'] })
      queryClient.invalidateQueries({ queryKey: ['applications', variables.id] })
      queryClient.invalidateQueries({ queryKey: ['stats'] })
    },
  })
}

// Delete application
export function useDeleteApplication() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async (id: number) => {
      await apiClient.delete(`/api/applications/${id}`)
      return id
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications'] })
      queryClient.invalidateQueries({ queryKey: ['stats'] })
    },
  })
}
