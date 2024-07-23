def from_http(url):
    import requests
    return requests.get(url).content

def from_ftp(url):
    import ftplib
    if not "://" in url:
        actual_host = url
    else:
        actual_host = url.split("://")[1]
    if "/" not in actual_host:
        raise ValueError("No file specified")
    file = actual_host.split("/")[-1]
    directory = "/".join(actual_host.split("/")[1:-1])
    if not directory:
        directory = "/"
    actual_host = actual_host.split("/")[0]
    with ftplib.FTP(actual_host) as ftp:
        ftp.login()
        ftp.cwd(directory)
        import io
        with io.BytesIO() as f:
            ftp.retrbinary("RETR " + file, f.write)
            return f.getvalue()