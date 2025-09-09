#!/bin/bash

# Copyright 2002-2024 CS GROUP
# Licensed to CS GROUP (CS) under one or more
# contributor license agreements.  See the NOTICE file distributed with
# this work for additional information regarding copyright ownership.
# CS licenses this file to You under the Apache License, Version 2.0
# (the "License"); you may not use this file except in compliance with
# the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# this script updates the following files in the orekit-data directory
#  UTC-TAI.history
#  Earth-Orientation-Parameters/IAU-1980/finals.all
#  Earth-Orientation-Parameters/IAU-2000/finals2000A.all
#  MSAFE/mmm####f10{""|_prd|-prd}.txt (where mmm is a month abbreviation and #### a year)
#  CSSI-Space-Weather-Data/SpaceWeather-All-v1.2.txt

# base URLS
usno_ser7_url=https://maia.usno.navy.mil/ser7
iers_rapid_url=https://datacenter.iers.org/data
msafe_url_uploads=https://www.nasa.gov/wp-content/uploads
msafe_url_atoms=https://www3.nasa.gov/sites/default/files/atoms/files
cssi_url=http://ftp.agi.com/pub/DynamicEarthData

# fetch a file from an URL
fetch_URL()
{ echo -n "fetching URL $1/$2… " 1>&2
  if [ -f "$2" ] ; then
      mv "$2" "$2.old"
  fi

  # convert either DOS or MAC file to Unix line endings
  if curl -s "$1/$2" | sed 's,\r$,,' | tr '\015' '\012' > "$2" && test -s "$2" ; then
      if [ -f "$2.old" ] ; then
          # remove old file
          rm "$2.old"
      fi
      echo "done" 1>&2
      return 0
  else
      if [ -f "$2" ] ; then
          # remove empty file
          rm "$2"
      fi
      if [ -f "$2.old" ] ; then
          # recover old file
          mv "$2.old" "$2"
      fi
      echo "failed!" 1>&2
      return 1
  fi

}

# fetch an MSAFE file
fetch_MSAFE()
{
    # when NASA website got a makover, some files have been published with variable URLs
    # which reference the update date, not the file content, so sometimes
    # files are slightly off, when not loaded exactly on the month they refer to
    # there are also files with an URL that does not refer to any date
    # the files contain a 7 lines header followed by data that starts 6 months before the
    # file date. In order to quickly check the file content, we look at line 1 to see if
    # it contains the header for "TABLE 3" and we check line 14 to see if it corresponds
    # to the expected year and month of the file
    local base_url name y14 m14
    for base_url in $(MSAFE_attempt_urls $1 $2) ; do
        for name in $(MSAFE_attempt_names $1 $2); do
            if fetch_URL ${base_url} ${name} ; then
                case $(head -1 $name | tr ' ' '_') in
                    __TABLE_3_*)
                        # it seems to be a regular MSAFE f10 file
                        # after a 7 lines header, data starts and the 6 first lines correspond to the past 6 months
                        # hence line 14 should match the year and month in the file name
                        read y14 m14 <<< $(sed -n '14s,^ *\([0-9]*\)\.[0-9]* *\([A-Z]*\).*$,\1 \2,p' $name | tr '[:upper:]' '[:lower:]';)
                        if [ "$y14" = "$1" ] && [ "$m14" = $(month_name "$2") ] ; then
                            echo 1>&2 "$name is complete"
                            return 0
                        else
                            rm $name
                            echo 1>&2 "$name removed (content does not match expected dates)"
                        fi ;;
                    *)
                        rm $name
                        echo 1>&2 "$name removed (fetched only a 404 error page)" ;;
                esac
            fi
        done
    done
    return 1
}

# month name
month_name()
{
    case $1 in
        '01') echo 'jan' ;;
        '02') echo 'feb' ;;
        '03') echo 'mar' ;;
        '04') echo 'apr' ;;
        '05') echo 'may' ;;
        '06') echo 'jun' ;;
        '07') echo 'jul' ;;
        '08') echo 'aug' ;;
        '09') echo 'sep' ;;
        '10') echo 'oct' ;;
        '11') echo 'nov' ;;
        '12') echo 'dec' ;;
        *) echo "unknown month $2" 1>&2 ; exit 1;;
    esac
}

# find the MSAFE year and month-number following an existing one
next_date()
{
    if [ $2 -gt 11 ] ; then
        echo $(($1 + 1)) "01"
    else
        echo "$1" $(($(echo $2 | sed 's,^0,,') + 1)) | sed 's,^\([0-9][0-9][0-9][0-9]\) \([0-9]\)$,\1 0\2,'
    fi
}

# find the MSAFE year and month-number preceding an existing one
previous_date()
{
    if [ $2 -gt 1 ] ; then
        echo "$1" $(($(echo $2 | sed 's,^0,,') - 1)) | sed 's,^\([0-9][0-9][0-9][0-9]\) \([0-9]\)$,\1 0\2,'
    else
        echo $(($1 - 1)) "12"
    fi
}

# build several attempts for MSAFE URLs
# we use the base year/month of the data to build the first tried URL,
# plus some previous and later dates in case the file is not uploaded at the right time
# some old files were also uploaded on 2019-04 or with a completely different
# base URL
MSAFE_attempt_urls()
{
    local yprev1 mprev1 yprev2 mprev2 yprev3 mprev3 yprev4 mprev4
    local ynext1 mnext1 ynext2 mnext2 ynext3 mnext3 ynext4 mnext4
    read  yprev1 mprev1 <<< $(previous_date $1      $2     )
    read  yprev2 mprev2 <<< $(previous_date $yprev1 $mprev1)
    read  yprev3 mprev3 <<< $(previous_date $yprev2 $mprev2)
    read  yprev4 mprev4 <<< $(previous_date $yprev3 $mprev3)
    read  ynext1 mnext1 <<< $(next_date     $1      $2     )
    read  ynext2 mnext2 <<< $(next_date     $ynext1 $mnext1)
    read  ynext3 mnext3 <<< $(next_date     $ynext2 $mnext2)
    read  ynext4 mnext4 <<< $(next_date     $ynext3 $mnext3)
    echo ${msafe_url_uploads}/$1/$2           \
         ${msafe_url_uploads}/$yprev1/$mprev1 \
         ${msafe_url_uploads}/$ynext1/$mnext1 \
         ${msafe_url_uploads}/$yprev2/$mprev2 \
         ${msafe_url_uploads}/$ynext2/$mnext2 \
         ${msafe_url_uploads}/$yprev3/$mprev3 \
         ${msafe_url_uploads}/$ynext3/$mnext3 \
         ${msafe_url_uploads}/$yprev4/$mprev4 \
         ${msafe_url_uploads}/$ynext4/$mnext4 \
         ${msafe_url_uploads}/2019/04         \
         ${msafe_url_atoms} 
}

# build several attempts for MSAFE names
MSAFE_attempt_names()
{
    local month=$(month_name $2)
    echo ${month}${1}f10-prd.txt \
         ${month}${1}f10_prd.txt \
         ${month}${1}f10.txt
}

# find the first MSAFE file that is missing in current directory
first_missing_MSAFE()
{
    local y=1999 m=03
    local found msafe
    while [ $y -le 9999 ] ; do
        found=0
        for msafe in $(MSAFE_attempt_names $y $m) ; do
            if [ -f "$msafe" ] ; then
                found=1
            fi
        done
        if [ $found -eq 0 ] ; then
            echo $y $m
            return 0
        else
            read y m <<< $(next_date $y $m)
        fi
    done
}

# update (overwriting) leap seconds file
fetch_URL $usno_ser7_url tai-utc.dat
 
# update (overwriting) Earth Orientation Parameters
(cd Earth-Orientation-Parameters/IAU-2000 && fetch_URL $iers_rapid_url/9 finals2000A.all)
(cd Earth-Orientation-Parameters/IAU-1980 && fetch_URL $iers_rapid_url/7 finals.all)

# update (adding files) Marshall Solar Activity Future Estimation
read fetch_year fetch_month <<< $(cd MSAFE ; first_missing_MSAFE)
while [ ! -z "$fetch_year" ] ; do
    if $(cd MSAFE ; fetch_MSAFE $fetch_year $fetch_month) ; then
        read fetch_year fetch_month <<< $(next_date $fetch_year $fetch_month)
    else
        fetch_year=""
        fetch_month=""
    fi
done

# update (overwriting) CSSI space weather data
(cd CSSI-Space-Weather-Data && fetch_URL $cssi_url SpaceWeather-All-v1.2.txt)
