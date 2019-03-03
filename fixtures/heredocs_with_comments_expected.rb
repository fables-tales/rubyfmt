HTML_HEADER = <<-EOF
<!DOCTYPE html>
<html lang='en'>
<head>
  <title>RSpec results</title>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
  <meta http-equiv="Expires" content="-1" />
  <meta http-equiv="Pragma" content="no-cache" />
  <style type="text/css">
  body {
    margin: 0;
    padding: 0;
    background: #fff;
    font-size: 80%;
  }
  </style>
  <script type="text/javascript">
    // <![CDATA[
#{GLOBAL_SCRIPTS}
    // ]]>
  </script>
  <style type="text/css">
#{GLOBAL_STYLES}
  </style>
</head>
<body>
EOF
