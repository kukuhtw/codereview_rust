-- phpMyAdmin SQL Dump
-- version 5.2.1
-- https://www.phpmyadmin.net/
--
-- Host: 127.0.0.1
-- Generation Time: Aug 18, 2025 at 07:56 AM
-- Server version: 10.4.32-MariaDB
-- PHP Version: 8.0.30

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `codereview2`
--

-- --------------------------------------------------------

--
-- Table structure for table `analysis`
--

CREATE TABLE `analysis` (
  `id` bigint(20) NOT NULL,
  `file_id` bigint(20) NOT NULL,
  `analisa_fungsi` mediumtext DEFAULT NULL,
  `analisa_relasi_file` mediumtext DEFAULT NULL,
  `analisa_relasi_db` mediumtext DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `applications`
--

CREATE TABLE `applications` (
  `id` bigint(20) NOT NULL,
  `nama_aplikasi` varchar(255) NOT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `app_summary`
--

CREATE TABLE `app_summary` (
  `id` bigint(20) NOT NULL,
  `app_id` bigint(20) NOT NULL,
  `summary` mediumtext DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `files`
--

CREATE TABLE `files` (
  `id` bigint(20) NOT NULL,
  `app_id` bigint(20) NOT NULL,
  `nama_file` varchar(512) NOT NULL,
  `nama_folder` varchar(1024) DEFAULT NULL,
  `full_path` varchar(2048) NOT NULL,
  `content_file` text DEFAULT NULL,
  `json_graph` mediumtext DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- --------------------------------------------------------

--
-- Table structure for table `file_metadata`
--

CREATE TABLE `file_metadata` (
  `id` bigint(20) NOT NULL,
  `file_id` bigint(20) NOT NULL,
  `line_count` int(11) DEFAULT NULL,
  `imports` text DEFAULT NULL,
  `sql_queries` text DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `analysis`
--
ALTER TABLE `analysis`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `uq_analysis_file` (`file_id`),
  ADD KEY `idx_analysis_file_id` (`file_id`);

--
-- Indexes for table `applications`
--
ALTER TABLE `applications`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `app_summary`
--
ALTER TABLE `app_summary`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `uq_summary_app` (`app_id`);

--
-- Indexes for table `files`
--
ALTER TABLE `files`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`),
  ADD KEY `idx_files_app_id` (`app_id`);

--
-- Indexes for table `file_metadata`
--
ALTER TABLE `file_metadata`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `uq_file_metadata_file` (`file_id`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `analysis`
--
ALTER TABLE `analysis`
  MODIFY `id` bigint(20) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `applications`
--
ALTER TABLE `applications`
  MODIFY `id` bigint(20) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3;

--
-- AUTO_INCREMENT for table `app_summary`
--
ALTER TABLE `app_summary`
  MODIFY `id` bigint(20) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `files`
--
ALTER TABLE `files`
  MODIFY `id` bigint(20) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=97;

--
-- AUTO_INCREMENT for table `file_metadata`
--
ALTER TABLE `file_metadata`
  MODIFY `id` bigint(20) NOT NULL AUTO_INCREMENT;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `analysis`
--
ALTER TABLE `analysis`
  ADD CONSTRAINT `fk_analysis_file` FOREIGN KEY (`file_id`) REFERENCES `files` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `app_summary`
--
ALTER TABLE `app_summary`
  ADD CONSTRAINT `fk_summary_app` FOREIGN KEY (`app_id`) REFERENCES `applications` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `files`
--
ALTER TABLE `files`
  ADD CONSTRAINT `fk_files_app` FOREIGN KEY (`app_id`) REFERENCES `applications` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `file_metadata`
--
ALTER TABLE `file_metadata`
  ADD CONSTRAINT `fk_meta_file` FOREIGN KEY (`file_id`) REFERENCES `files` (`id`) ON DELETE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
