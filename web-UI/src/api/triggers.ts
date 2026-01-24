import { useMutation } from '@tanstack/react-query'
import { apiClient } from './client'

// Trigger search scraper
export function useTriggerSearch() {
  return useMutation({
    mutationFn: async () => {
      const { data } = await apiClient.post('/api/trigger/search')
      return data
    },
  })
}

// Trigger schedule
export function useTriggerSchedule() {
  return useMutation({
    mutationFn: async () => {
      const { data } = await apiClient.post('/api/trigger/schedule')
      return data
    },
  })
}

// Trigger track
export function useTriggerTrack() {
  return useMutation({
    mutationFn: async () => {
      const { data } = await apiClient.post('/api/trigger/track')
      return data
    },
  })
}
