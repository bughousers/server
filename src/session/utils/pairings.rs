// Copyright (C) 2020  Yakup Koray Budanaz

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::VecDeque;

pub fn create_pairings(n: usize) -> VecDeque<((u8, u8), (u8, u8))> {
    //every element has its number
    let mut arr = Vec::with_capacity(n);
    for i in 0..n {
        arr.push((i + 1) as u8);
    }
    //get all possible n -> 4 distributions
    //let mut v = combinations(&mut arr,4);
    //get all possible team combinations
    let mut teams = combinations(&mut arr, 2);
    //rm ordering from tuples
    teams = rm_ordering(&mut teams);
    //create team pairintgs
    //meaning tuples (a,b)(c,d) where a,b,c,d are unique
    let mut x = create_team_pairings(&teams);
    let y = team_comb(&mut x);
    y
}

pub fn create_team_pairings(v: &Vec<Vec<u8>>) -> VecDeque<[u8; 4]> {
    let mut tmp = VecDeque::with_capacity(4 * v.len());
    for i in v.iter() {
        for j in v.iter() {
            if (j[0] != i[0] && j[0] != i[1]) && (j[1] != i[1] && j[1] != i[0]) {
                tmp.push_back([i[0], i[1], j[0], j[1]]);
            }
        }
    }
    tmp
}

pub fn map_indices(v: &Vec<Vec<u8>>) -> Vec<usize> {
    let mut indices = Vec::new();
    for i in 0..v.len() {
        match &(v[i][0..2]) {
            [a, b] => {
                let f = *a;
                let s = *b;
                match contains_vec(&v, [s, f].to_vec()) {
                    None => {}
                    Some(i) => {
                        indices.push(i);
                    }
                }
            }
            _ => {
                println!("lel");
                return Vec::new();
            }
        }
    }
    indices
}

pub fn rm_ordering(v: &mut Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    // println!("1");
    let mut x = map_indices(v);
    let mut rmed = Vec::with_capacity(x.len());
    let mut checked = Vec::with_capacity(x.len());
    for i in 0..x.len() {
        rmed.push(false);
        checked.push(false);
    }
    //println!("{:?}",x);
    //println!("2");
    let mut i = 0;
    while i < x.len() {
        //if x[0] -> 3 then x[3] -> 0
        //rm the second one
        if !checked[i] {
            if !rmed[x[i]] && !checked[x[i]] {
                rmed[x[i]] = true;
                checked[i] = true;
                checked[x[i]] = true;
            }
        }

        i += 1;
    }
    //println!("{:?}",x);

    let mut only_needed = Vec::new();
    i = 0;
    for i in 0..x.len() {
        if !rmed[x[i]] {
            only_needed.push(copy(&v[x[i]]));
        }
    } //println!("{:?}",only_needed);

    only_needed
}

//l is the elements, n is the group size :: n == depth of the tree
pub fn combinations(l: &mut Vec<u8>, n: usize) -> Vec<Vec<u8>> {
    let len = l.len();

    //create a tree that has e -> len many children
    // -> len -1 many children -> until 1

    //calculate vector length
    let mut x = 1;
    for o in 0..n {
        x *= (len - o);
    }

    let mut vec: Vec<Vec<u8>> = Vec::new();
    for m in 0..x {
        vec.push(Vec::new());
        for _ in 0..n {
            vec[m].push(0);
        }
    }
    //println!("{}",x);

    //iteration for the first depth
    let mut blocksize = x / len;
    let mut blocks = len;
    //println!("blocksize:{},blocks:{}",blocksize,blocks);

    let mut de = depthEnum::new();

    for i in 0..blocks {
        //stride n -> n-1 -> n-2
        for k in 0..blocksize {
            //fill (1,...),(1,...) then (2,....)(2....) and so on for first depth
            let mut tp = &mut vec[i * blocksize + k];
            tp[0] = l[i];
        }
    }

    de.depth = n;
    de.depthctr = 1;

    //for depth 1 ans so on ..
    //max depth ==

    for j in 1..n {
        for k in 0..x {
            //fill (1,...),(1,...) then (2,....)(2....) and so on for first depth
            let mut tp = &mut vec[k];
            //get the least available number for the next depth
            tp[j] = enum_w_depth(copy(&l), &tp, &de);

            de.offset += 1;
            if de.offset >= len - de.depthctr {
                de.offset = 0;
            }
        }
        de.depthctr += 1;
    }

    vec
}

pub struct depthEnum {
    pub depth: usize,
    offset: usize,
    depthctr: usize,
}

impl depthEnum {
    pub fn new() -> Self {
        depthEnum {
            depth: 0,
            offset: 0,
            depthctr: 0,
        }
    }
}

//l = list of possible numbers
//
//tp = current vector to check for values
pub fn enum_w_depth(mut l: Vec<u8>, tp: &Vec<u8>, de: &depthEnum) -> u8 {
    diff_values(&mut l, tp);
    l.sort();
    //must be n (depth) - de.depthctr many elements left
    l[de.offset]
}

pub fn diff_values(v1: &mut Vec<u8>, v2: &Vec<u8>) {
    for i in 0..v2.len() {
        match contains(&v1, v2[i]) {
            None => {}
            Some(e) => {
                v1.remove(e);
            }
        }
    }
}

pub fn copy(v1: &Vec<u8>) -> Vec<u8> {
    let mut v2 = Vec::with_capacity(v1.len());
    for i in 0..v1.len() {
        v2.push(v1[i]);
    }
    v2
}

pub fn contains(v1: &Vec<u8>, el: u8) -> Option<usize> {
    for i in 0..v1.len() {
        if v1[i] == el {
            return Some(i);
        }
    }
    return None;
}

pub fn contains_usize(v1: &Vec<usize>, el: usize) -> bool {
    for i in 0..v1.len() {
        if v1[i] == el {
            return true;
        }
    }
    return false;
}

//vector has to have the length 2
pub fn contains_vec(v1: &Vec<Vec<u8>>, el: Vec<u8>) -> Option<usize> {
    for i in 0..v1.len() {
        if v1[i][0] == el[0] && v1[i][1] == el[1] {
            return Some(i);
        }
    }
    return None;
}

//low is the least integer in vec
//high is the highest integer in vec
//the vector needs to have all integers from [low,high];
/*
pub fn order_as_teams(mut vec: &mut Vec<Vec<u8>>, low:u8, high:u8) -> VecDeque<((u8,u8),(u8,u8))> {

}
*/

//returns all matches for the given teams: 1:(a,b) and 2:(c,d)
pub fn team_comb(vec: &mut VecDeque<[u8; 4]>) -> VecDeque<((u8, u8), (u8, u8))> {
    let mut l = VecDeque::with_capacity(4 * vec.len());

    for i in 0..vec.len() {
        let a = vec[i][0];
        let b = vec[i][1];
        let c = vec[i][2];
        let d = vec[i][3];

        l.push_back(((a, b), (c, d)));
        l.push_back(((a, b), (d, c)));
        l.push_back(((b, a), (c, d)));
        l.push_back(((b, a), (d, c)));
    }
    l
}

pub fn print_v(x: &VecDeque<((u8, u8), (u8, u8))>) {
    for i in 0..x.len() {
        print!("[");

        print!("({},", (x[i].0).0);
        print!("{}),", (x[i].0).1);

        print!("({},", (x[i].1).0);
        print!("{})", (x[i].1).1);

        println!("]");
    }
}
