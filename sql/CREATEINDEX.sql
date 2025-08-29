CREATE INDEX idx_files_app_id ON files(app_id);
CREATE INDEX idx_files_search ON files (app_id, nama_file(191), nama_folder(191), full_path(191));
