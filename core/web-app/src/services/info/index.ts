import { useQuery } from '@tanstack/react-query';
import { DataLayer } from '@/infra/data-layer';

function useInfo() {
  return useQuery({
    queryKey: ['info'],
    queryFn: async () => {
      // Get the info from the server
      const { data } = await DataLayer.get('/api/info');
      return data;
    },
    staleTime: Infinity,
  });
}

export { useInfo };
