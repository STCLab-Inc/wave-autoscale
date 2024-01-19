import { QueryClient } from '@tanstack/react-query';

// Create a client
const queryClient = new QueryClient();

export function getQueryClient() {
  return queryClient;
}

export function invalidateQuery(queryKey: string[]) {
  return queryClient.invalidateQueries({
    queryKey,
  });
}
