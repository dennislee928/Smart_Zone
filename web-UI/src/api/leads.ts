import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'
import type { Lead, ApiResponse } from '../types'

// Fetch all leads
export function useLeads(filters?: { status?: string; bucket?: string; search?: string }) {
  return useQuery({
    queryKey: ['leads', filters],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (filters?.status) params.append('status', filters.status)
      if (filters?.bucket) params.append('bucket', filters.bucket)
      if (filters?.search) params.append('search', filters.search)
      
      const { data } = await apiClient.get<ApiResponse<Lead>>(`/api/leads?${params.toString()}`)
      return (data.leads || []) as Lead[]
    },
  })
}

// Fetch single lead
export function useLead(id: number) {
  return useQuery({
    queryKey: ['leads', id],
    queryFn: async () => {
      const { data } = await apiClient.get<ApiResponse<Lead>>(`/api/leads/${id}`)
      return data.lead as Lead
    },
    enabled: !!id,
  })
}

// Create lead
export function useCreateLead() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async (lead: Omit<Lead, 'id' | 'createdAt' | 'updatedAt'>) => {
      const { data } = await apiClient.post<ApiResponse<Lead>>('/api/leads', lead)
      return data.lead as Lead
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['leads'] })
    },
  })
}

// Update lead
export function useUpdateLead() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async ({ id, ...updates }: Partial<Lead> & { id: number }) => {
      const { data } = await apiClient.put<ApiResponse<Lead>>(`/api/leads/${id}`, updates)
      return data.lead as Lead
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['leads'] })
      queryClient.invalidateQueries({ queryKey: ['leads', variables.id] })
    },
  })
}

// Delete lead
export function useDeleteLead() {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async (id: number) => {
      await apiClient.delete(`/api/leads/${id}`)
      return id
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['leads'] })
    },
  })
}
