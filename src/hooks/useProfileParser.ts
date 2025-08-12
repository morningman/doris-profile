import { useState, useCallback } from 'react';
import { ProfileParser } from '../utils/profileParser';
import type { ParsedProfileData } from '../types/profile';

export const useProfileParser = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const parseProfile = useCallback(async (file: File): Promise<ParsedProfileData | null> => {
    setIsLoading(true);
    setError(null);

    try {
      const text = await file.text();
      const parser = new ProfileParser();
      const result = parser.parse(text);
      
      if (result.hasErrors) {
        setError(result.errors?.join(', ') || 'Parse failed');
        return null;
      }
      
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const generateCompleteJson = useCallback(async (file: File): Promise<any | null> => {
    setIsLoading(true);
    setError(null);

    try {
      const text = await file.text();
      const parser = new ProfileParser();
      const result = parser.generateCompleteJson(text);
      
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    parseProfile,
    generateCompleteJson,
    isLoading,
    error,
  };
};