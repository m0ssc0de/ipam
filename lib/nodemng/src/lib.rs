#[macro_use]
extern crate cmd_lib;
extern crate ippool;
use std::path::Path;

#[derive(Debug)]
pub enum NodeError {
    IPErrorCreat(ippool::IpPoolError),
    IPErrorEmpty,
    CrtErrorPathNotExist,
    CrtErrorCreat(std::io::Error),
}

impl From<std::io::Error> for NodeError {
    fn from(err: std::io::Error) -> NodeError {
        NodeError::CrtErrorCreat(err)
    }
}

pub struct NodeMNG<'a> {
    cacrt_path: &'a Path,
    cakey_path: &'a Path,
    cfg_path: &'a Path,
    ip_pool: ippool::IpPool,
}

impl<'a> NodeMNG<'a> {
    pub fn new(
        cacrt_path: &'a Path,
        cakey_path: &'a Path,
        cfg_path: &'a Path,
        ip_pool_op: Option<ippool::IpPool>,
    ) -> Result<NodeMNG<'a>, NodeError> {
        if !cacrt_path.exists() || !cakey_path.exists() || !cfg_path.exists() {
            return Err(NodeError::CrtErrorPathNotExist);
        }

        let ip_pool = match ip_pool_op {
            Some(ipp) => ipp,
            None => match "192.168.0.2/24".parse() {
                Ok(ip) => match ippool::IpPool::new(ip, 1) {
                    Ok(net) => net,
                    Err(e) => return Err(NodeError::IPErrorCreat(e)),
                },
                Err(_) => panic!("can not pars ip"),
            },
        };
        Ok(NodeMNG {
            cacrt_path,
            cakey_path,
            cfg_path,
            ip_pool,
        })
    }
    pub fn get_node(&mut self, name: &str) -> Result<String, NodeError> {
        // run_cmd!("nebula-cert ca -name \"Myorganization, Inc\"");
        let ip = match self.ip_pool.new_addr() {
            Some(ip) => ip,
            None => return Err(NodeError::IPErrorEmpty),
        };
        let _ = run_cmd!("rm -rf {}", name);
        let _ = run_cmd!("mkdir -p {}", name);
        run_cmd!(
            "nebula-cert sign -ca-crt {} -ca-key {} -name {} -ip {} -out-crt ./{}/host.crt -out-key ./{}/host.key",
            self.cacrt_path.to_string_lossy(),
            self.cakey_path.to_string_lossy(),
            name,
            ip,
            name,
            name
        )?;
        run_cmd!(
            // "cat {} > ./{}/config.yml",
            "cp {} {}/",
            self.cfg_path.to_string_lossy(),
            name
        )?;
        match run_fun!("tar -zcf - {} | base64 -w 0", name) {
            Ok(s) => Ok(s),
            Err(e) => Err(NodeError::CrtErrorCreat(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let crt = std::path::Path::new("./ca.crt");
        let key = std::path::Path::new("./ca.key");
        let cfg = std::path::Path::new("./config.yml");
        let mut mng = match crate::NodeMNG::new(crt, key, cfg, None) {
            Ok(m) => m,
            Err(e) => panic!(e),
        };

        match mng.get_node("123") {
            Ok(s) => assert_eq!("ok", s),
            Err(e) => panic!(e),
        }
        assert_eq!(2 + 2, 4);
    }
}
