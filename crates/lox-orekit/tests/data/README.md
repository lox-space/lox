This project contains some example data files useful that may be used
as an initial setup for the [Orekit library](https://www.orekit.org/).

In order to use these files, simply download the
[latest archive](https://gitlab.orekit.org/orekit/orekit-data/-/archive/main/orekit-data-main.zip)
and unzip it anywhere you want. Rename the `orekit-data-main` folder to
`orekit-data`, note the path of this folder and add the following lines at
the start of your program:

```java
File orekitData = new File("/path/to/the/folder/orekit-data");
DataProvidersManager manager = DataContext.getDefault().getDataProvidersManager();
manager.addProvider(new DirectoryCrawler(orekitData));
```

This zip file contains:

* JPL DE 440 ephemerides from 1990 to 2149 (more accurate than DE441 for current dates),
* IERS Earth orientation parameters from 1973 to May 2025
  with predicted data up to late 2024 (both IAU-1980 and IAU-2000),
* configuration data for ITRF versions used in regular IERS files,
* leap seconds history from 1972 to mid	2025,
* Marshall Solar Activity Future Estimation from 1999 to June 2025,
* CSSI Space Weather Data with observed data from 1957 to May 2025
  with predicted data up to 22 years in the future
* the Eigen 6S gravity field
* the FES 2004 ocean tides model.

The provided archive is just a convenience, it is intended as a starting
point for Orekit setup. Users are responsible to update the files in
the unzipped folder to suit their needs as new data is published by IERS,
NASA... This is why we suggest to rename `orekit-data-main`
into `orekit-data` to show that the live data folder is decorrelated
from the initial main folder.

The update.sh script is a bash script that allows to update the data
automatically by uploading individual files from the sites of the
laboratories that generate them, regardless of these files being
updated in Orekit git repository. It is in fact the script the Orekit
team uses to update the repository.

There is *NO* guarantee that this convenience archive will be updated
or even provided in the future.

## Notes for Orekit Python users

You can download this orekit data repository via pip (don't forget to activate your
conda environment containing orekit beforehand):

```bash
pip install git+https://gitlab.orekit.org/orekit/orekit-data.git
```

This will install a Python library named `orekitdata`. The Python library receives a new
version tag at every commit in this repository so that your Python project can pull the
newest version of the orekit data every time this repository is updated. The 7 last
characters of the version tag correspond to the 7 first characters of the git hash of the
commit: for instance the Python version `orekitdata-0+untagged.70.g91b2c79` correspond to
the commit of the `main` branch of this repository with a githash starting with `91b2c79`.

Then to pass the data folder to Orekit in Python, follow the instructions in the
[Python wrapper Wiki](https://gitlab.orekit.org/orekit-labs/python-wrapper/-/wikis/installation#physical-data).

