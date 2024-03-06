import { errorToast } from '@/utils/toast';
import axios from 'axios';

const instance = axios.create({
  baseURL: '',
  timeout: 3000,
  headers: {},
});

instance.interceptors.response.use(
  (res) => res,
  (error) => {
    if (error.response.status === 500) {
      // errorToast(
      //   'Server Connection Error. Check your connection and try again.'
      // );
    }
  }
);

export { instance as DataLayer };
