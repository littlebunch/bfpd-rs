-- MariaDB dump 10.17  Distrib 10.4.13-MariaDB, for osx10.15 (x86_64)
--
-- Host: localhost    Database: bfpd5
-- ------------------------------------------------------
-- Server version	10.4.13-MariaDB

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `__diesel_schema_migrations`
--

DROP TABLE IF EXISTS `__diesel_schema_migrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `__diesel_schema_migrations` (
  `version` varchar(50) NOT NULL,
  `run_on` timestamp NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`version`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `brands`
--

DROP TABLE IF EXISTS `brands`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `brands` (
  `id` int(10) NOT NULL AUTO_INCREMENT,
  `owner` varchar(255) NOT NULL DEFAULT '',
  `brand` varchar(255) DEFAULT NULL,
  `subbrand` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `brand_owner_IDX` (`owner`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=112427 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `derivations`
--

DROP TABLE IF EXISTS `derivations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `derivations` (
  `id` int(10) NOT NULL,
  `code` varchar(255) NOT NULL,
  `description` mediumtext NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `food_groups`
--

DROP TABLE IF EXISTS `food_groups`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `food_groups` (
  `id` int(10) NOT NULL AUTO_INCREMENT,
  `description` varchar(255) NOT NULL DEFAULT '',
  PRIMARY KEY (`id`),
  KEY `food_groups_description_IDX` (`description`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=995 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `foods`
--

DROP TABLE IF EXISTS `foods`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `foods` (
  `id` int(10) NOT NULL AUTO_INCREMENT,
  `publication_date` datetime NOT NULL,
  `modified_date` datetime NOT NULL,
  `available_date` datetime NOT NULL,
  `upc` varchar(24) NOT NULL,
  `fdc_id` varchar(24) NOT NULL,
  `description` mediumtext NOT NULL,
  `food_group_id` int(10) NOT NULL DEFAULT 0,
  `brand_id` int(10) NOT NULL DEFAULT 0,
  `datasource` varchar(8) NOT NULL,
  `serving_size` double DEFAULT NULL,
  `serving_unit` varchar(24) DEFAULT NULL,
  `serving_description` varchar(256) DEFAULT NULL,
  `country` varchar(24) DEFAULT NULL,
  `ingredients` mediumtext DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `foods_fdc_id_IDX` (`fdc_id`) USING BTREE,
  KEY `foods_upc_IDX` (`upc`) USING BTREE,
  KEY `foods_FK` (`brand_id`),
  KEY `foods_description_IDX` (`description`(768)) USING BTREE,
  KEY `foods_brand_id_IDX` (`brand_id`) USING BTREE,
  KEY `foods_food_group_id_IDX` (`food_group_id`) USING BTREE,
  KEY `foods_country_IDX` (`country`) USING BTREE,
  CONSTRAINT `foods_FK` FOREIGN KEY (`brand_id`) REFERENCES `brands` (`id`),
  CONSTRAINT `foods_FK_1` FOREIGN KEY (`food_group_id`) REFERENCES `food_groups` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=5107548 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `nutrient_data`
--

DROP TABLE IF EXISTS `nutrient_data`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `nutrient_data` (
  `id` int(10) NOT NULL AUTO_INCREMENT,
  `value` double NOT NULL DEFAULT 0,
  `portion_value` double NOT NULL DEFAULT 0,
  `standard_error` double DEFAULT NULL,
  `minimum` double DEFAULT NULL,
  `maximum` double DEFAULT NULL,
  `median` double DEFAULT NULL,
  `derivation_id` int(10) NOT NULL DEFAULT 0,
  `nutrient_id` int(10) NOT NULL DEFAULT 0,
  `food_id` int(10) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `nutrient_data_food_id_IDX` (`food_id`) USING BTREE,
  KEY `nutrient_data_FK` (`nutrient_id`),
  KEY `nutrient_data_FK_1` (`derivation_id`),
  CONSTRAINT `nutrient_data_FK` FOREIGN KEY (`nutrient_id`) REFERENCES `nutrients` (`id`),
  CONSTRAINT `nutrient_data_FK_1` FOREIGN KEY (`derivation_id`) REFERENCES `derivations` (`id`),
  CONSTRAINT `nutrient_data_food_FK` FOREIGN KEY (`food_id`) REFERENCES `foods` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=54709615 DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `nutrients`
--

DROP TABLE IF EXISTS `nutrients`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `nutrients` (
  `id` int(10) NOT NULL,
  `nutrientno` varchar(12) NOT NULL,
  `description` varchar(255) NOT NULL,
  `unit` varchar(24) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `nutrientno` (`nutrientno`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2021-12-19  8:04:44
