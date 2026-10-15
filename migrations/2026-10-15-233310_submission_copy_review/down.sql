UPDATE submissions SET status = 'Pending' WHERE status = 'CopyReview';
UPDATE submission_history SET status = 'Pending' WHERE status = 'CopyReview';